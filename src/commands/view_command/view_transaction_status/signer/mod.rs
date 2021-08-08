use dialoguer::Input;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliSendFrom {
    /// Specify a signer
    Signer(CliSender),
}

#[derive(Debug, Clone)]
pub enum SendFrom {
    Signer(Sender),
}

impl CliSendFrom {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::Signer(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("signer".to_owned());
                args
            }
        }
    }
}

impl From<SendFrom> for CliSendFrom {
    fn from(send_from: SendFrom) -> Self {
        match send_from {
            SendFrom::Signer(sender) => Self::Signer(sender.into()),
        }
    }
}

impl From<CliSendFrom> for SendFrom {
    fn from(item: CliSendFrom) -> Self {
        match item {
            CliSendFrom::Signer(cli_sender) => {
                let sender = Sender::from(cli_sender);
                Self::Signer(sender)
            }
        }
    }
}

impl SendFrom {
    pub fn send_from() -> Self {
        Self::from(CliSendFrom::Signer(Default::default()))
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
        transaction_hash: String,
    ) -> crate::CliResult {
        match self {
            SendFrom::Signer(sender) => {
                sender
                    .process(network_connection_config, transaction_hash)
                    .await
            }
        }
    }
}

/// Specify the account that signed the transaction
#[derive(Debug, Default, Clone, clap::Clap)]
pub struct CliSender {
    pub account_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Sender {
    pub account_id: String,
}

impl CliSender {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = std::collections::VecDeque::new();
        if let Some(account_id) = &self.account_id {
            args.push_front(account_id.to_string());
        }
        args
    }
}

impl From<Sender> for CliSender {
    fn from(sender: Sender) -> Self {
        Self {
            account_id: Some(sender.account_id),
        }
    }
}

impl From<CliSender> for Sender {
    fn from(item: CliSender) -> Self {
        let account_id: String = match item.account_id {
            Some(cli_account_id) => cli_account_id,
            None => Sender::input_sender_account_id(),
        };
        Self { account_id }
    }
}

impl Sender {
    pub fn input_sender_account_id() -> String {
        println!();
        Input::new()
            .with_prompt("Specify the account that signed the transaction")
            .interact_text()
            .unwrap()
    }

    fn rpc_client(&self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
        transaction_hash: String,
    ) -> crate::CliResult {
        let account_id = self.account_id.clone();
        let query_view_transaction_status = self
            .rpc_client(network_connection_config.archival_rpc_url().as_str())
            .tx(transaction_hash, account_id)
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to fetch query for view transaction: {:?}",
                    err
                ))
            })?;
        println!("Transactiion status: {:#?}", query_view_transaction_status);
        Ok(())
    }
}
