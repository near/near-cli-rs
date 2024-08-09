extern crate dirs;

use std::str::FromStr;

use color_eyre::eyre::{ContextCompat, WrapErr};
use inquire::{CustomType, Select};
use near_primitives::transaction::TransactionV0;

use crate::common::JsonRpcClientExt;
use crate::common::RpcQueryResponseExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = SignLegacyKeychainContext)]
pub struct SignLegacyKeychain {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    signer_public_key: Option<crate::types::public_key::PublicKey>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    nonce: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub block_hash: Option<crate::types::crypto_hash::CryptoHash>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub block_height: Option<near_primitives::types::BlockHeight>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    meta_transaction_valid_for: Option<u64>,
    #[interactive_clap(subcommand)]
    submit: super::Submit,
}

#[derive(Clone)]
pub struct SignLegacyKeychainContext {
    pub(crate) network_config: crate::config::NetworkConfig,
    pub(crate) global_context: crate::GlobalContext,
    pub(crate) signed_transaction_or_signed_delegate_action:
        super::SignedTransactionOrSignedDelegateAction,
    pub(crate) on_before_sending_transaction_callback:
        crate::transaction_signature_options::OnBeforeSendingTransactionCallback,
    pub(crate) on_after_sending_transaction_callback:
        crate::transaction_signature_options::OnAfterSendingTransactionCallback,
}

impl SignLegacyKeychainContext {
    #[tracing::instrument(
        name = "Signing the transaction with a key saved in legacy keychain ...",
        skip_all
    )]
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignLegacyKeychain as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let network_config = previous_context.network_config.clone();

        let file_name = format!(
            "{}.json",
            &previous_context.prepopulated_transaction.signer_id
        );
        let mut path =
            std::path::PathBuf::from(&previous_context.global_context.config.credentials_home_dir);

        let data_path: std::path::PathBuf = {
            let dir_name = network_config.network_name.clone();
            path.push(&dir_name);

            if previous_context.global_context.offline {
                path.push(
                    previous_context
                        .prepopulated_transaction
                        .signer_id
                        .to_string(),
                );
                path.push(&format!(
                    "{}.json",
                    scope
                        .signer_public_key
                        .clone()
                        .wrap_err(
                            "Signer public key is required to sign a transaction in offline mode"
                        )?
                        .to_string()
                        .replace(':', "_")
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
                    let mut path = std::path::PathBuf::from(
                        &previous_context.global_context.config.credentials_home_dir,
                    );
                    path.push(dir_name);
                    path.push(
                        previous_context
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
        let data = std::fs::read_to_string(&data_path).wrap_err_with(|| {
            format!(
                "Access key file for account <{}> on network <{}> not found!",
                previous_context.prepopulated_transaction.signer_id, network_config.network_name
            )
        })?;
        let account_json: super::AccountKeyPair = serde_json::from_str(&data)
            .wrap_err_with(|| format!("Error reading data from file: {:?}", &data_path))?;

        let (nonce, block_hash, block_height) = if previous_context.global_context.offline {
            (
                scope
                    .nonce
                    .wrap_err("Nonce is required to sign a transaction in offline mode")?,
                scope
                    .block_hash
                    .wrap_err("Block Hash is required to sign a transaction in offline mode")?
                    .0,
                scope
                    .block_height
                    .wrap_err("Block Height is required to sign a transaction in offline mode")?,
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

        let mut unsigned_transaction =
            near_primitives::transaction::Transaction::V0(TransactionV0 {
                public_key: account_json.public_key.clone(),
                block_hash,
                nonce,
                signer_id: previous_context.prepopulated_transaction.signer_id,
                receiver_id: previous_context.prepopulated_transaction.receiver_id,
                actions: previous_context.prepopulated_transaction.actions,
            });

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
                global_context: previous_context.global_context,
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
            global_context: previous_context.global_context,
            signed_transaction_or_signed_delegate_action: signed_transaction.into(),
            on_before_sending_transaction_callback: previous_context
                .on_before_sending_transaction_callback,
            on_after_sending_transaction_callback: previous_context
                .on_after_sending_transaction_callback,
        })
    }
}

impl From<SignLegacyKeychainContext> for super::SubmitContext {
    fn from(item: SignLegacyKeychainContext) -> Self {
        Self {
            network_config: item.network_config,
            global_context: item.global_context,
            signed_transaction_or_signed_delegate_action: item
                .signed_transaction_or_signed_delegate_action,
            on_before_sending_transaction_callback: item.on_before_sending_transaction_callback,
            on_after_sending_transaction_callback: item.on_after_sending_transaction_callback,
        }
    }
}

impl SignLegacyKeychain {
    fn input_signer_public_key(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::public_key::PublicKey>> {
        if context.global_context.offline {
            let network_config = context.network_config.clone();

            let mut path =
                std::path::PathBuf::from(&context.global_context.config.credentials_home_dir);

            let dir_name = network_config.network_name;
            path.push(&dir_name);

            path.push(context.prepopulated_transaction.signer_id.to_string());

            let signer_dir = path.read_dir()?;

            let key_list = signer_dir
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| entry.file_name().into_string().ok())
                .filter(|file_name_str| file_name_str.starts_with("ed25519_"))
                .map(|file_name_str| file_name_str.replace(".json", "").replace('_', ":"))
                .collect::<Vec<_>>();

            let selected_input = Select::new("Choose public_key:", key_list).prompt()?;

            return Ok(Some(crate::types::public_key::PublicKey::from_str(
                &selected_input,
            )?));
        }
        Ok(None)
    }

    fn input_nonce(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<u64>> {
        if context.global_context.offline {
            return Ok(Some(
                CustomType::<u64>::new("Enter a nonce for the access key:").prompt()?,
            ));
        }
        Ok(None)
    }

    fn input_block_hash(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::crypto_hash::CryptoHash>> {
        if context.global_context.offline {
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
    ) -> color_eyre::eyre::Result<Option<near_primitives::types::BlockHeight>> {
        if context.global_context.offline {
            return Ok(Some(
                CustomType::<near_primitives::types::BlockHeight>::new(
                    "Enter recent block height:",
                )
                .prompt()?,
            ));
        }
        Ok(None)
    }
}
