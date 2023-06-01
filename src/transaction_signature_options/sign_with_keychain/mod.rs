extern crate dirs;

use std::str::FromStr;

use color_eyre::eyre::WrapErr;
use inquire::{CustomType, Select};

use crate::common::JsonRpcClientExt;
use crate::common::RpcQueryResponseExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = SignKeychainContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignKeychain {
    #[allow(dead_code)]
    #[interactive_clap(skip)]
    signer_public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    nonce: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub block_hash: Option<crate::types::crypto_hash::CryptoHash>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub block_height: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    meta_transaction_valid_for: Option<u64>,
    #[interactive_clap(subcommand)]
    submit: super::Submit,
}

#[derive(Clone)]
pub struct SignKeychainContext {
    network_config: crate::config::NetworkConfig,
    offline: bool,
    signed_transaction_or_signed_delegate_action: super::SignedTransactionOrSignedDelegateAction,
    on_before_sending_transaction_callback:
        crate::transaction_signature_options::OnBeforeSendingTransactionCallback,
    on_after_sending_transaction_callback:
        crate::transaction_signature_options::OnAfterSendingTransactionCallback,
}

impl SignKeychainContext {
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignKeychain as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let network_config = previous_context.network_config.clone();

        let file_name = format!(
            "{}.json",
            &previous_context.prepopulated_transaction.signer_id
        );
        let mut path = std::path::PathBuf::from(&previous_context.config.credentials_home_dir);

        let data_path: std::path::PathBuf = {
            let dir_name = network_config.network_name.clone();
            path.push(&dir_name);

            if previous_context.offline {
                path.push(
                    previous_context
                        .prepopulated_transaction
                        .signer_id
                        .to_string(),
                );
                path.push(&format!(
                    "{}.json",
                    scope.signer_public_key.to_string().replace(':', "_")
                ));
                path
            } else {
                path.push(file_name);
                if path.exists() {
                    path
                } else {
                    let access_key_list = network_config
                        .json_rpc_client()
                        .blocking_call_view_access_key_list(
                            &previous_context.prepopulated_transaction.signer_id,
                            near_primitives::types::Finality::Final.into(),
                        )
                        .wrap_err_with(|| {
                            format!(
                                "Failed to fetch access KeyList for {}",
                                previous_context.prepopulated_transaction.signer_id
                            )
                        })?
                        .access_key_list_view()?;
                    let mut path =
                        std::path::PathBuf::from(&previous_context.config.credentials_home_dir);
                    path.push(dir_name);
                    path.push(
                        &previous_context
                            .prepopulated_transaction
                            .signer_id
                            .to_string(),
                    );
                    let mut data_path = std::path::PathBuf::new();
                    'outer: for access_key in access_key_list.keys {
                        let account_public_key = access_key.public_key.to_string();
                        let is_full_access_key: bool = match &access_key.access_key.permission {
                            near_primitives::views::AccessKeyPermissionView::FullAccess => true,
                            near_primitives::views::AccessKeyPermissionView::FunctionCall {
                                allowance: _,
                                receiver_id: _,
                                method_names: _,
                            } => false,
                        };
                        let dir = path
                        .read_dir()
                        .wrap_err("There are no access keys found in the keychain for the signer account. Log in before signing transactions with keychain.")?;
                        for entry in dir {
                            if let Ok(entry) = entry {
                                if entry
                                    .path()
                                    .file_stem()
                                    .unwrap()
                                    .to_str()
                                    .unwrap()
                                    .contains(account_public_key.rsplit(':').next().unwrap())
                                    && is_full_access_key
                                {
                                    data_path.push(entry.path());
                                    break 'outer;
                                }
                            } else {
                                return Err(color_eyre::Report::msg(
                                "There are no access keys found in the keychain for the signer account. Log in before signing transactions with keychain."
                            ));
                            };
                        }
                    }
                    data_path
                }
            }
        };
        let data = std::fs::read_to_string(&data_path).wrap_err("Access key file not found!")?;
        let account_json: super::AccountKeyPair = serde_json::from_str(&data)
            .wrap_err_with(|| format!("Error reading data from file: {:?}", &data_path))?;

        let (nonce, block_hash, block_height) = if previous_context.offline {
            (
                scope.nonce.unwrap(),
                scope.block_hash.unwrap().0,
                scope.block_height.unwrap(),
            )
        } else {
            let rpc_query_response = network_config
                .json_rpc_client()
                .blocking_call_view_access_key(
                    &previous_context.prepopulated_transaction.signer_id,
                    &account_json.public_key,
                    near_primitives::types::BlockReference::latest()
                )
                .wrap_err(
                    "Cannot sign a transaction due to an error while fetching the most recent nonce value",
                )?;
            (
                rpc_query_response
                    .access_key_view()
                    .wrap_err("Error current_nonce")?
                    .nonce
                    + 1,
                rpc_query_response.block_hash,
                rpc_query_response.block_height,
            )
        };

        let mut unsigned_transaction = near_primitives::transaction::Transaction {
            public_key: account_json.public_key.clone(),
            block_hash,
            nonce,
            signer_id: previous_context.prepopulated_transaction.signer_id.clone(),
            receiver_id: previous_context.prepopulated_transaction.receiver_id,
            actions: previous_context.prepopulated_transaction.actions,
        };

        (previous_context.on_before_signing_callback)(&mut unsigned_transaction, &network_config)?;

        if network_config.meta_transaction_relayer_url.is_some() {
            let max_block_height = block_height
                + scope
                    .meta_transaction_valid_for
                    .unwrap_or(super::META_TRANSACTION_VALID_FOR_DEFAULT);

            let signed_delegate_action = super::get_signed_delegate_action(
                unsigned_transaction,
                &account_json.public_key,
                account_json.private_key,
                max_block_height,
            );

            return Ok(Self {
                network_config: previous_context.network_config,
                offline: previous_context.offline,
                signed_transaction_or_signed_delegate_action: signed_delegate_action.into(),
                on_before_sending_transaction_callback: previous_context
                    .on_before_sending_transaction_callback,
                on_after_sending_transaction_callback: previous_context
                    .on_after_sending_transaction_callback,
            });
        }

        let signature = account_json
            .private_key
            .sign(unsigned_transaction.get_hash_and_size().0.as_ref());

        let signed_transaction = near_primitives::transaction::SignedTransaction::new(
            signature.clone(),
            unsigned_transaction,
        );

        eprintln!("\nYour transaction was signed successfully.");
        eprintln!("Public key: {}", account_json.public_key);
        eprintln!("Signature: {}", signature);

        Ok(Self {
            network_config: previous_context.network_config,
            offline: previous_context.offline,
            signed_transaction_or_signed_delegate_action: signed_transaction.into(),
            on_before_sending_transaction_callback: previous_context
                .on_before_sending_transaction_callback,
            on_after_sending_transaction_callback: previous_context
                .on_after_sending_transaction_callback,
        })
    }
}

impl From<SignKeychainContext> for super::SubmitContext {
    fn from(item: SignKeychainContext) -> Self {
        Self {
            network_config: item.network_config,
            offline: item.offline,
            signed_transaction_or_signed_delegate_action: item
                .signed_transaction_or_signed_delegate_action,
            on_before_sending_transaction_callback: item.on_before_sending_transaction_callback,
            on_after_sending_transaction_callback: item.on_after_sending_transaction_callback,
        }
    }
}

impl interactive_clap::FromCli for SignKeychain {
    type FromCliContext = crate::commands::TransactionContext;
    type FromCliError = color_eyre::eyre::Error;
    fn from_cli(
        optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();

        let signer_public_key: crate::types::public_key::PublicKey = if context.offline {
            let network_config = context.network_config.clone();

            let mut path = std::path::PathBuf::from(&context.config.credentials_home_dir);

            let dir_name = network_config.network_name;
            path.push(&dir_name);

            path.push(context.prepopulated_transaction.signer_id.to_string());

            let signer_dir = match path.read_dir() {
                Ok(dir) => dir,
                Err(err) => {
                    return interactive_clap::ResultFromCli::Err(Some(clap_variant), err.into())
                }
            };

            let key_list = signer_dir
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| entry.file_name().into_string().ok())
                .filter(|file_name_str| file_name_str.starts_with("ed25519_"))
                .map(|file_name_str| file_name_str.replace(".json", "").replace('_', ":"))
                .collect::<Vec<_>>();

            let selected_input = match Select::new("Choose public_key:", key_list).prompt() {
                Ok(selected) => selected,
                Err(err) => {
                    return interactive_clap::ResultFromCli::Err(Some(clap_variant), err.into())
                }
            };

            match crate::types::public_key::PublicKey::from_str(&selected_input) {
                Ok(public_key) => public_key,
                Err(err) => {
                    return interactive_clap::ResultFromCli::Err(Some(clap_variant), err.into())
                }
            }
        } else {
            near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519).into()
        };

        if clap_variant.nonce.is_none() {
            clap_variant.nonce = match Self::input_nonce(&context) {
                Ok(optional_nonce) => optional_nonce,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let nonce = clap_variant.nonce;
        if clap_variant.block_hash.is_none() {
            clap_variant.block_hash = match Self::input_block_hash(&context) {
                Ok(optional_block_hash) => optional_block_hash,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let block_hash = clap_variant.block_hash;
        if clap_variant.block_height.is_none() {
            clap_variant.block_height = match Self::input_block_height(&context) {
                Ok(optional_block_height) => optional_block_height,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let block_height = clap_variant.block_height;
        if clap_variant.meta_transaction_valid_for.is_none() {
            clap_variant.meta_transaction_valid_for =
                match Self::input_meta_transaction_valid_for(&context) {
                    Ok(meta_transaction_valid_for) => meta_transaction_valid_for,
                    Err(err) => {
                        return interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
                    }
                };
        }
        let meta_transaction_valid_for = clap_variant.meta_transaction_valid_for;

        let new_context_scope = InteractiveClapContextScopeForSignKeychain {
            signer_public_key,
            nonce,
            block_hash,
            block_height,
            meta_transaction_valid_for,
        };
        let output_context =
            match SignKeychainContext::from_previous_context(context, &new_context_scope) {
                Ok(new_context) => new_context,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };

        match super::Submit::from_cli(clap_variant.submit.take(), output_context.into()) {
            interactive_clap::ResultFromCli::Ok(cli_submit) => {
                clap_variant.submit = Some(cli_submit);
                interactive_clap::ResultFromCli::Ok(clap_variant)
            }
            interactive_clap::ResultFromCli::Cancel(optional_cli_submit) => {
                clap_variant.submit = optional_cli_submit;
                interactive_clap::ResultFromCli::Cancel(Some(clap_variant))
            }
            interactive_clap::ResultFromCli::Back => interactive_clap::ResultFromCli::Back,
            interactive_clap::ResultFromCli::Err(optional_cli_submit, err) => {
                clap_variant.submit = optional_cli_submit;
                interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
            }
        }
    }
}

impl SignKeychain {
    fn input_nonce(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<u64>> {
        if context.offline {
            return Ok(Some(
                CustomType::<u64>::new("Enter a nonce for the access key:").prompt()?,
            ));
        }
        Ok(None)
    }

    fn input_block_hash(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::crypto_hash::CryptoHash>> {
        if context.offline {
            return Ok(Some(
                CustomType::<crate::types::crypto_hash::CryptoHash>::new(
                    "Enter recent block hash:",
                )
                .prompt()?,
            ));
        }
        Ok(None)
    }

    fn input_block_height(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<u64>> {
        if context.offline {
            return Ok(Some(
                CustomType::<u64>::new("Enter recent block height:").prompt()?,
            ));
        }
        Ok(None)
    }

    fn input_meta_transaction_valid_for(
        _context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<u64>> {
        Ok(None)
    }
}
