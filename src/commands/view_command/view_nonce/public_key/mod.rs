use dialoguer::Input;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliAccessKey {
    /// Specify public key
    PublicKey(CliAccessKeyType),
}

#[derive(Debug, Clone)]
pub enum AccessKey {
    PublicKey(AccessKeyType),
}

impl CliAccessKey {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::PublicKey(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("public-key".to_owned());
                args
            }
        }
    }
}

impl From<AccessKey> for CliAccessKey {
    fn from(access_key: AccessKey) -> Self {
        match access_key {
            AccessKey::PublicKey(access_key_type) => Self::PublicKey(access_key_type.into()),
        }
    }
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
        account_id: near_primitives::types::AccountId,
        network_connection_config: crate::common::ConnectionConfig,
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
#[derive(Debug, Default, Clone, clap::Clap)]
pub struct CliAccessKeyType {
    public_key: Option<near_crypto::PublicKey>,
}

#[derive(Debug, Clone)]
pub struct AccessKeyType {
    pub public_key: near_crypto::PublicKey,
}

impl CliAccessKeyType {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = std::collections::VecDeque::new();
        if let Some(public_key) = &self.public_key {
            args.push_front(public_key.to_string());
        }
        args
    }
}

impl From<AccessKeyType> for CliAccessKeyType {
    fn from(access_key_type: AccessKeyType) -> Self {
        Self {
            public_key: access_key_type.public_key.into(),
        }
    }
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
        account_id: near_primitives::types::AccountId,
        network_connection_config: crate::common::ConnectionConfig,
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
