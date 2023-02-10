extern crate dirs;
use near_primitives::borsh::BorshSerialize;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = super::SubmitContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignKeychain {
    #[interactive_clap(skip)]
    signer_public_key: crate::types::public_key::PublicKey,
    // #[interactive_clap(skip)]
    // signer_private_key: crate::types::secret_key::SecretKey,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    nonce: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    block_hash: Option<String>,
    #[interactive_clap(subcommand)]
    submit: super::Submit,
}

#[derive(Debug, Clone)]
pub struct SignKeychainContext {
    config: crate::config::Config,
    network_config: crate::config::NetworkConfig,
    prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    signer_public_key: crate::types::public_key::PublicKey,
    // signer_private_key: crate::types::secret_key::SecretKey,
    nonce: Option<u64>,
    block_hash: Option<String>,
    signed_transaction: near_primitives::transaction::SignedTransaction,
    base64_transaction: String,
}

impl SignKeychainContext {
    pub async fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignKeychain as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Result<Self, color_eyre::eyre::Error> {
        // let network_name = &context.network_name.clone().expect("Failed to get network name");
        // let networks = &context.config.networks;
        // let network_config = networks
        //     .get(network_name)
        //     .expect("Impossible to get network name!")
        //     .clone();
        let network_config = previous_context.network_config.clone();

        let file_name = format!("{}.json", &previous_context.transaction.signer_id);
        let mut path = std::path::PathBuf::from(&previous_context.config.credentials_home_dir);

        let data_path: std::path::PathBuf = {
            let dir_name = network_config.network_name.clone();
            path.push(&dir_name);
            path.push(file_name);

            if path.exists() {
                path
            } else {
                let query_view_method_response = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(
                        network_config.json_rpc_client().call(
                            near_jsonrpc_client::methods::query::RpcQueryRequest {
                                block_reference: near_primitives::types::Finality::Final.into(),
                                request: near_primitives::views::QueryRequest::ViewAccessKeyList {
                                    account_id: previous_context.transaction.signer_id.clone(),
                                },
                            },
                        ), // .await
                    )
                    .map_err(|err| {
                        color_eyre::Report::msg(format!(
                            "Failed to fetch query for view key list: {:?}",
                            err
                        ))
                    })?;
                let access_key_list =
                    if let near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKeyList(
                        result,
                    ) = query_view_method_response.kind
                    {
                        result
                    } else {
                        return Err(color_eyre::Report::msg("Error call result".to_string()));
                    };
                let mut path =
                    std::path::PathBuf::from(&previous_context.config.credentials_home_dir);
                path.push(dir_name);
                path.push(&previous_context.transaction.signer_id.to_string());
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
                                .map_err(|err| {
                                    color_eyre::Report::msg(format!("There are no access keys found in the keychain for the signer account. Log in before signing transactions with keychain. {}", err))
                                })?;
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
                                        "There are no access keys found in the keychain for the signer account. Log in before signing transactions with keychain.".to_string()));
                        };
                    }
                }
                data_path
            }
        };
        let data = std::fs::read_to_string(data_path).map_err(|err| {
            color_eyre::Report::msg(format!("Access key file not found! Error: {}", err))
        })?;
        let account_json: super::AccountKeyPair = serde_json::from_str(&data)
            .map_err(|err| color_eyre::Report::msg(format!("Error reading data: {}", err)))?;

        let signer_public_key =
            crate::types::public_key::PublicKey(account_json.public_key.clone());
        // let signer_private_key = crate::types::secret_key::SecretKey(account_json.private_key);

        let online_signer_access_key_response = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(network_config.json_rpc_client().call(
                near_jsonrpc_client::methods::query::RpcQueryRequest {
                    block_reference: near_primitives::types::Finality::Final.into(),
                    request: near_primitives::views::QueryRequest::ViewAccessKey {
                        account_id: previous_context.transaction.signer_id.clone(),
                        public_key: account_json.public_key.clone(),
                    },
                },
            ))
            .map_err(|err| {
                // println!("\nUnsigned transaction:\n");
                // crate::common::print_transaction(prepopulated_unsigned_transaction.clone());
                println!("\nYour transaction was not successfully signed.\n");
                color_eyre::Report::msg(format!(
                    "Failed to fetch public key information for nonce: {:?}",
                    err
                ))
            })?;
        let current_nonce =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKey(
                online_signer_access_key,
            ) = online_signer_access_key_response.kind
            {
                online_signer_access_key.nonce
            } else {
                return Err(color_eyre::Report::msg("Error current_nonce".to_string()));
            };

        let mut unsigned_transaction = near_primitives::transaction::Transaction {
            public_key: account_json.public_key.clone(),
            block_hash: online_signer_access_key_response.block_hash,
            nonce: current_nonce + 1,
            ..previous_context.transaction.clone()
        };

        (previous_context.on_before_signing_callback)(&mut unsigned_transaction)?;

        let signature = account_json
            .private_key
            .sign(unsigned_transaction.get_hash_and_size().0.as_ref());
        let signed_transaction = near_primitives::transaction::SignedTransaction::new(
            signature.clone(),
            unsigned_transaction,
        );

        (previous_context.on_after_signing_callback)(&signed_transaction)?;

        let base64_transaction = near_primitives::serialize::to_base64(
            signed_transaction
                .try_to_vec()
                .expect("Transaction is not expected to fail on serialization"),
        );
        println!("\nYour transaction was signed successfully.");
        println!("Public key: {}", account_json.public_key);
        println!("Signature: {}", signature);

        Ok(Self {
            config: previous_context.config,
            network_config: previous_context.network_config,
            prepopulated_unsigned_transaction: previous_context.transaction,
            signer_public_key,
            // signer_private_key: scope.signer_private_key.clone(),
            nonce: scope.nonce,
            block_hash: scope.block_hash.clone(),
            signed_transaction: signed_transaction,
            base64_transaction: base64_transaction,
        })
    }
}

impl From<SignKeychainContext> for super::SubmitContext {
    fn from(item: SignKeychainContext) -> Self {
        Self {
            network_config: item.network_config,
            signed_transaction: item.signed_transaction.into(),
            base64_transaction: item.base64_transaction,
        }
    }
}

impl interactive_clap::FromCli for SignKeychain {
    type FromCliContext = crate::commands::TransactionContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<SignKeychain as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> Result<Option<Self>, Self::FromCliError>
    where
        Self: Sized + interactive_clap::ToCli,
    {
        // from_cli EXAMPLE: print unsigned transaction (context)

        // macro-generated:

        let nonce: Option<u64> = optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.nonce);
        let block_hash: Option<String> = optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.block_hash.clone());

        let new_context_scope = InteractiveClapContextScopeForSignKeychain {
            signer_public_key: crate::types::public_key::PublicKey(near_crypto::PublicKey::empty(
                near_crypto::KeyType::ED25519,
            )),
            // signer_private_key: signer_private_key.clone(),
            nonce,
            block_hash: block_hash.clone(),
        };
        let keychain_context = tokio::runtime::Runtime::new().unwrap().block_on(
            SignKeychainContext::from_previous_context(context.clone(), &new_context_scope),
        )?;
        let new_context = super::SubmitContext::from(keychain_context.clone());

        let optional_submit = super::Submit::from_cli(
            optional_clap_variant.and_then(|clap_variant| clap_variant.submit),
            new_context,
        )?;
        let submit = if let Some(submit) = optional_submit {
            submit
        } else {
            return Ok(None);
        };

        Ok(Some(Self {
            signer_public_key: keychain_context.signer_public_key,
            // signer_private_key,
            nonce,
            block_hash,
            submit,
        }))
    }
}

impl SignKeychain {
    pub fn get_signer_public_key(&self) -> near_crypto::PublicKey {
        self.signer_public_key.clone().into()
    }

    pub async fn process(
        &self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_config: crate::config::NetworkConfig,
        credentials_home_dir: std::path::PathBuf,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        // let file_name = format!("{}.json", prepopulated_unsigned_transaction.signer_id);
        // let mut path = std::path::PathBuf::from(&credentials_home_dir);

        // let data_path: std::path::PathBuf = {
        //     let dir_name = network_config.network_name.as_str();
        //     path.push(dir_name);
        //     path.push(file_name);

        //     if path.exists() {
        //         path
        //     } else {
        //         let query_view_method_response = network_config
        //             .json_rpc_client()
        //             .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
        //                 block_reference: near_primitives::types::Finality::Final.into(),
        //                 request: near_primitives::views::QueryRequest::ViewAccessKeyList {
        //                     account_id: prepopulated_unsigned_transaction.signer_id.clone(),
        //                 },
        //             })
        //             .await
        //             .map_err(|err| {
        //                 color_eyre::Report::msg(format!(
        //                     "Failed to fetch query for view key list: {:?}",
        //                     err
        //                 ))
        //             })?;
        //         let access_key_list =
        //             if let near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKeyList(
        //                 result,
        //             ) = query_view_method_response.kind
        //             {
        //                 result
        //             } else {
        //                 return Err(color_eyre::Report::msg("Error call result".to_string()));
        //             };
        //         let mut path = std::path::PathBuf::from(&credentials_home_dir);
        //         path.push(dir_name);
        //         path.push(&prepopulated_unsigned_transaction.signer_id.to_string());
        //         let mut data_path = std::path::PathBuf::new();
        //         'outer: for access_key in access_key_list.keys {
        //             let account_public_key = access_key.public_key.to_string();
        //             let is_full_access_key: bool = match &access_key.access_key.permission {
        //                 near_primitives::views::AccessKeyPermissionView::FullAccess => true,
        //                 near_primitives::views::AccessKeyPermissionView::FunctionCall {
        //                     allowance: _,
        //                     receiver_id: _,
        //                     method_names: _,
        //                 } => false,
        //             };
        //             let dir = path
        //                     .read_dir()
        //                     .map_err(|err| {
        //                         color_eyre::Report::msg(format!("There are no access keys found in the keychain for the signer account. Log in before signing transactions with keychain. {}", err))
        //                     })?;
        //             for entry in dir {
        //                 if let Ok(entry) = entry {
        //                     if entry
        //                         .path()
        //                         .file_stem()
        //                         .unwrap()
        //                         .to_str()
        //                         .unwrap()
        //                         .contains(account_public_key.rsplit(':').next().unwrap())
        //                         && is_full_access_key
        //                     {
        //                         data_path.push(entry.path());
        //                         break 'outer;
        //                     }
        //                 } else {
        //                     return Err(color_eyre::Report::msg(
        //                             "There are no access keys found in the keychain for the signer account. Log in before signing transactions with keychain.".to_string()));
        //                 };
        //             }
        //         }
        //         data_path
        //     }
        // };
        // let data = std::fs::read_to_string(data_path).map_err(|err| {
        //     color_eyre::Report::msg(format!("Access key file not found! Error: {}", err))
        // })?;
        // let account_json: super::AccountKeyPair = serde_json::from_str(&data)
        //     .map_err(|err| color_eyre::Report::msg(format!("Error reading data: {}", err)))?;
        // let sign_with_private_key = super::sign_with_private_key::SignPrivateKey {
        //     signer_public_key: crate::types::public_key::PublicKey(account_json.public_key),
        //     signer_private_key: crate::types::secret_key::SecretKey(account_json.private_key),
        //     nonce: self.nonce,
        //     block_hash: self.block_hash.clone(),
        //     submit: self.submit.clone(),
        // };
        // sign_with_private_key
        //     .process(prepopulated_unsigned_transaction, network_config)
        //     .await

        //self.submit.process(network_config, self.signed_transaction.clone().into(), self.base64_transaction.clone()).await
        Ok(None)
    }
}
