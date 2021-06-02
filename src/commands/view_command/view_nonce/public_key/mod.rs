use dialoguer::Input;

#[derive(Debug, clap::Clap)]
pub enum CliAccessKey {
    /// Specify public key
    PublicKey(CliAccessKeyType),
}

#[derive(Debug)]
pub enum AccessKey {
    PublicKey(AccessKeyType),
}

impl From<CliAccessKey> for AccessKey {
    fn from(item: CliAccessKey) -> Self {
        match item {
            CliAccessKey::PublicKey(cli_delete_access_key_type) => {
                Self::PublicKey(cli_delete_access_key_type.into())
            }
        }
    }
}

impl AccessKey {
    pub fn choose_key() -> Self {
        Self::from(CliAccessKey::PublicKey(Default::default()))
    }

    pub async fn process(
        self,
        account_id: String,
        network_connection_config: super::operation_mode::online_mode::select_server::ConnectionConfig,
    ) -> crate::CliResult {
        match self {
            AccessKey::PublicKey(access_key_type) => {
                access_key_type
                    .process(account_id, network_connection_config)
                    .await
            }
        }
    }
}

/// Specify the access key to be deleted
#[derive(Debug, Default, clap::Clap)]
pub struct CliAccessKeyType {
    public_key: Option<near_crypto::PublicKey>,
}

#[derive(Debug)]
pub struct AccessKeyType {
    pub public_key: near_crypto::PublicKey,
}

impl From<CliAccessKeyType> for AccessKeyType {
    fn from(item: CliAccessKeyType) -> Self {
        let public_key: near_crypto::PublicKey = match item.public_key {
            Some(cli_public_key) => cli_public_key,
            None => AccessKeyType::input_public_key(),
        };
        Self { public_key }
    }
}

impl AccessKeyType {
    pub fn input_public_key() -> near_crypto::PublicKey {
        Input::new()
            .with_prompt("Enter a public key for this access key")
            .interact_text()
            .unwrap()
    }

    fn rpc_client(self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    pub async fn process(
        self,
        account_id: String,
        network_connection_config: super::operation_mode::online_mode::select_server::ConnectionConfig,
    ) -> crate::CliResult {
        let public_key = self.public_key.clone();
        let online_signer_access_key_response = self
            .rpc_client(network_connection_config.rpc_url().as_str())
            .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::ViewAccessKey {
                    account_id,
                    public_key: public_key.clone(),
                },
            })
            .await
            .map_err(|err| {
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
                return Err(color_eyre::Report::msg(format!("Error current_nonce")));
            };
        println!(
            "\ncurrent nonce: {}  for a public key: {}",
            current_nonce, public_key
        );
        Ok(())
    }
}
