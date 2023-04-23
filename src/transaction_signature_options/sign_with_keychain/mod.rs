extern crate dirs;

use std::str::FromStr;

use color_eyre::eyre::WrapErr;

use crate::common::JsonRpcClientExt;
use crate::common::RpcQueryResponseExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = SignMacosKeychainContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignKeychain {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    nonce: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    block_hash: Option<String>,
    #[interactive_clap(subcommand)]
    submit: super::Submit,
}

#[derive(Clone)]
pub struct SignKeychainContext {
    network_config: crate::config::NetworkConfig,
    signed_transaction: near_primitives::transaction::SignedTransaction,
    on_before_sending_transaction_callback:
        crate::transaction_signature_options::OnBeforeSendingTransactionCallback,
    on_after_sending_transaction_callback:
        crate::transaction_signature_options::OnAfterSendingTransactionCallback,
}

impl SignKeychainContext {
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        _scope: &<SignKeychain as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
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
        };
        let data = std::fs::read_to_string(&data_path).wrap_err("Access key file not found!")?;
        let account_json: super::AccountKeyPair = serde_json::from_str(&data)
            .wrap_err_with(|| format!("Error reading data from file: {:?}", &data_path))?;

        let rpc_query_response = network_config
            .json_rpc_client()
            .blocking_call_view_access_key(
                &previous_context.prepopulated_transaction.signer_id,
                &account_json.public_key,
                near_primitives::types::BlockReference::latest(),
            )
            .wrap_err(
                "Cannot sign a transaction due to an error while fetching the most recent nonce value",
            )?;
        let current_nonce = rpc_query_response
            .access_key_view()
            .wrap_err("Error current_nonce")?
            .nonce;

        let mut unsigned_transaction = near_primitives::transaction::Transaction {
            public_key: account_json.public_key.clone(),
            block_hash: rpc_query_response.block_hash,
            nonce: current_nonce + 1,
            signer_id: previous_context.prepopulated_transaction.signer_id.clone(),
            receiver_id: previous_context.prepopulated_transaction.receiver_id,
            actions: previous_context.prepopulated_transaction.actions,
        };

        (previous_context.on_before_signing_callback)(&mut unsigned_transaction, &network_config)?;

        let signature = account_json
            .private_key
            .sign(unsigned_transaction.get_hash_and_size().0.as_ref());
        let signed_transaction = near_primitives::transaction::SignedTransaction::new(
            signature.clone(),
            unsigned_transaction.clone(),
        );
        // ============================================delegate action===========================================================================
        println!(
            "########### meta_transaction_relayer_url: {:?}",
            &network_config.meta_transaction_relayer_url
        );
        if let Some(meta_transaction_relayer_url) = &network_config.meta_transaction_relayer_url {
            use near_crypto::InMemorySigner;
            use near_primitives::borsh::BorshSerialize;
            use near_primitives::signable_message::{SignableMessage, SignableMessageType};
            use near_primitives::types::{BlockId, BlockReference};

            println!("##################");
            let unsigned_tx_copy = unsigned_transaction.clone();
            let signer = Some(InMemorySigner::from_secret_key(
                unsigned_tx_copy.signer_id.clone(),
                account_json.private_key.clone(),
            ));

            let block_header = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(network_config.json_rpc_client().call(
                    near_jsonrpc_client::methods::block::RpcBlockRequest {
                        block_reference: BlockReference::from(BlockId::Hash(
                            unsigned_transaction.block_hash.clone(),
                        )),
                    },
                ))?
                .header;
            let max_block_height = block_header.height + 100; // TODO is 100 blocks appropriate?

            let actions = unsigned_transaction
                .actions
                .iter()
                .map(|a| {
                    near_primitives::delegate_action::NonDelegateAction::try_from(a.clone())
                        .unwrap()
                })
                .collect();
            let delegate_action = near_primitives::delegate_action::DelegateAction {
                sender_id: unsigned_transaction.signer_id,
                receiver_id: unsigned_transaction.receiver_id,
                actions,
                nonce: unsigned_transaction.nonce,
                max_block_height,
                public_key: unsigned_transaction.public_key,
            };
            // create a new signature here signing the delegate action + discriminant
            let signable =
                SignableMessage::new(&delegate_action, SignableMessageType::DelegateAction);
            let signature = signable.sign(&signer.unwrap());
            let signed_delegate_action = near_primitives::delegate_action::SignedDelegateAction {
                delegate_action,
                signature,
            };

            let client = reqwest::blocking::Client::new();
            let payload = signed_delegate_action.try_to_vec().unwrap(); // serialize signed_delegate_action using borsh
            let json_payload = serde_json::to_vec(&payload).unwrap();
            let relayer_response = client
                .post(meta_transaction_relayer_url.clone())
                // .json(&data)
                .header("Content-Type", "application/json")
                .body(json_payload)
                .send()?;
            println!("############# relayer_response{:#?}", relayer_response);

            // ==========================================Sign delegate action with test_fro.testnet===================================

            let public_key = near_crypto::PublicKey::from_str(
                "ed25519:CCwvhsp3Y3BfLbfYJQJqXJA2CaSP7CRjn1t7PyEtsjej",
            )?;
            let secret_key = near_crypto::SecretKey::from_str("ed25519:3fLXqk4vjREgm7HhcobS1q6bf8ouCzqbAjHrZukscqatSDE3q9Rs8Cp8J7hv63bfXdQRgX7qaavKXAc77pmuMJod")?;
            let signer_id: near_primitives::types::AccountId = "test_fro.testnet".parse().unwrap();

            let rpc_query_response = network_config
            .json_rpc_client()
            .blocking_call_view_access_key(
                &signer_id,
                &public_key,
                near_primitives::types::BlockReference::latest(),
            )
            .wrap_err(
                "Cannot sign a transaction due to an error while fetching the most recent nonce value",
            )?;
            let current_nonce = rpc_query_response
                .access_key_view()
                .wrap_err("Error current_nonce")?
                .nonce;

            let actions = vec![near_primitives::transaction::Action::Delegate(
                signed_delegate_action,
            )];

            let unsigned_transaction = near_primitives::transaction::Transaction {
                public_key,
                block_hash: rpc_query_response.block_hash,
                nonce: current_nonce + 1,
                signer_id,
                receiver_id: previous_context.prepopulated_transaction.signer_id,
                actions,
            };

            let signature = secret_key
                .sign(unsigned_transaction.get_hash_and_size().0.as_ref());
            let signed_transaction = near_primitives::transaction::SignedTransaction::new(
                signature.clone(),
                unsigned_transaction.clone(),
            );


            eprintln!("\nYour transaction (delegate) was signed successfully.");
            eprintln!("{:#?}", signed_transaction);


            return Ok(Self {
                network_config: previous_context.network_config,
                signed_transaction,
                on_before_sending_transaction_callback: previous_context
                    .on_before_sending_transaction_callback,
                on_after_sending_transaction_callback: previous_context
                    .on_after_sending_transaction_callback,
            });
        }

        // =======================================================================================================================
        eprintln!("\nYour transaction was signed successfully.");
        eprintln!("Public key: {}", account_json.public_key);
        eprintln!("Signature: {}", signature);

        Ok(Self {
            network_config: previous_context.network_config,
            signed_transaction,
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
            signed_transaction: item.signed_transaction,
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
        let block_hash = clap_variant.block_hash.clone();

        let new_context_scope = InteractiveClapContextScopeForSignKeychain { nonce, block_hash };
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
    pub fn input_nonce(
        _context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<u64>> {
        Ok(None)
    }

    pub fn input_block_hash(
        _context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        Ok(None)
    }
}
