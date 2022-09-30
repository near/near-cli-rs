extern crate dirs;

use serde::Deserialize;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignKeychain {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    nonce: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    block_hash: Option<String>,
    #[interactive_clap(subcommand)]
    submit: Option<super::Submit>,
}

#[derive(Debug, Deserialize)]
struct User {
    public_key: near_crypto::PublicKey,
    private_key: near_crypto::SecretKey,
}

impl SignKeychain {
    pub fn from_cli(
        optional_clap_variant: Option<<SignKeychain as interactive_clap::ToCli>::CliVariant>,
        _context: crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<Self>> {
        let submit: Option<super::Submit> = optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.submit);
        Ok(Some(Self {
            nonce: None,
            block_hash: None,
            submit,
        }))
    }
}

impl SignKeychain {
    pub async fn process(
        &self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_config: crate::config::NetworkConfig,
        credentials_home_dir: std::path::PathBuf,
    ) -> crate::CliResult {
        let file_name = format!("{}.json", prepopulated_unsigned_transaction.signer_id);
        let mut path = std::path::PathBuf::from(&credentials_home_dir);

        let data_path: std::path::PathBuf = {
            let dir_name = network_config.network_name.as_str();
            path.push(dir_name);
            path.push(file_name);

            if path.exists() {
                path
            } else {
                let query_view_method_response = network_config
                    .json_rpc_client()?
                    .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                        block_reference: near_primitives::types::Finality::Final.into(),
                        request: near_primitives::views::QueryRequest::ViewAccessKeyList {
                            account_id: prepopulated_unsigned_transaction.signer_id.clone(),
                        },
                    })
                    .await
                    .map_err(|err| {
                        color_eyre::Report::msg(format!(
                            "Failed to fetch query for view key list: {:?}",
                            err
                        ))
                    })?;
                let access_key_view =
                    if let near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKeyList(
                        result,
                    ) = query_view_method_response.kind
                    {
                        result
                    } else {
                        return Err(color_eyre::Report::msg(format!("Error call result")));
                    };
                let mut path = std::path::PathBuf::from(&credentials_home_dir);
                path.push(dir_name);
                path.push(&prepopulated_unsigned_transaction.signer_id.to_string());
                let mut data_path = std::path::PathBuf::new();
                'outer: for access_key in access_key_view.keys {
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
                            return Err(color_eyre::Report::msg(format!(
                                    "There are no access keys found in the keychain for the signer account. Log in before signing transactions with keychain."
                                )));
                        };
                    }
                }
                data_path
            }
        };
        let data = std::fs::read_to_string(data_path).map_err(|err| {
            color_eyre::Report::msg(format!("Access key file not found! Error: {}", err))
        })?;
        let account_json: User = serde_json::from_str(&data)
            .map_err(|err| color_eyre::Report::msg(format!("Error reading data: {}", err)))?;
        let sign_with_private_key = super::sign_with_private_key::SignPrivateKey {
            signer_public_key: crate::types::public_key::PublicKey(account_json.public_key),
            signer_private_key: crate::types::secret_key::SecretKey(account_json.private_key),
            nonce: self.nonce.clone(),
            block_hash: self.block_hash.clone(),
            submit: self.submit.clone(),
        };
        sign_with_private_key
            .process(prepopulated_unsigned_transaction, network_config)
            .await
    }
}
