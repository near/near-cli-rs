use dialoguer::Input;

#[derive(Debug, clap::Clap)]
pub enum CliSendTo {
    /// Specify an account
    Account(CliSigner),
}

#[derive(Debug)]
pub enum SendTo {
    Account(Signer),
}

impl From<CliSendTo> for SendTo {
    fn from(item: CliSendTo) -> Self {
        match item {
            CliSendTo::Account(cli_signer) => {
                let signer = Signer::from(cli_signer);
                Self::Account(signer)
            }
        }
    }
}

impl SendTo {
    pub fn send_to() -> Self {
        Self::from(CliSendTo::Account(Default::default()))
    }

    pub async fn process(self, selected_server_url: url::Url) -> crate::CliResult {
        match self {
            SendTo::Account(signer) => signer.process(selected_server_url).await,
        }
    }
}

/// Specify signer to view the nonce for public key
#[derive(Debug, Default, clap::Clap)]
pub struct CliSigner {
    #[clap(long)]
    account_id: Option<String>,
    #[clap(long)]
    public_key: Option<near_crypto::PublicKey>,
}

#[derive(Debug)]
pub struct Signer {
    account_id: String,
    public_key: near_crypto::PublicKey,
}

impl From<CliSigner> for Signer {
    fn from(item: CliSigner) -> Self {
        let account_id: String = match item.account_id {
            Some(cli_account_id) => cli_account_id,
            None => Signer::input_account_id(),
        };
        let public_key: near_crypto::PublicKey = match item.public_key {
            Some(cli_public_key) => cli_public_key,
            None => Signer::input_public_key(),
        };
        Self {
            account_id,
            public_key,
        }
    }
}

impl Signer {
    fn input_account_id() -> String {
        println!();
        Input::new()
            .with_prompt("Enter your account ID")
            .interact_text()
            .unwrap()
    }

    fn input_public_key() -> near_crypto::PublicKey {
        Input::new()
            .with_prompt("Enter the public key")
            .interact_text()
            .unwrap()
    }

    fn rpc_client(self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    pub async fn process(self, selected_server_url: url::Url) -> crate::CliResult {
        let account_id = self.account_id.clone();
        let public_key = self.public_key.clone();
        let online_signer_access_key_response = self
            .rpc_client(&selected_server_url.as_str())
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
