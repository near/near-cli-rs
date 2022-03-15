use dialoguer::Input;
use std::str::FromStr;

/// previously set up RPC-server
#[derive(Debug, Default, Clone, clap::Clap)]
pub struct CliServer {
    #[clap(subcommand)]
    send: Option<CliSend>,
}

/// data for a custom server
#[derive(Debug, Default, Clone, clap::Clap)]
pub struct CliCustomServer {
    #[clap(long)]
    pub url: Option<crate::common::AvailableRpcServerUrl>,
    #[clap(subcommand)]
    send: Option<CliSend>,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub connection_config: crate::common::ConnectionConfig,
    send: Send,
}

impl CliCustomServer {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .send
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(url) = &self.url {
            args.push_front(url.to_string());
            args.push_front("--url".to_string());
        }
        args
    }
}

impl From<Server> for CliCustomServer {
    fn from(server: Server) -> Self {
        Self {
            url: Some(
                crate::common::AvailableRpcServerUrl::from_str(
                    server.connection_config.rpc_url().as_str(),
                )
                .unwrap(),
            ),
            send: Some(server.send.into()),
        }
    }
}

impl CliServer {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        self.send
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default()
    }
}

impl From<Server> for CliServer {
    fn from(server: Server) -> Self {
        Self {
            send: Some(server.send.into()),
        }
    }
}

impl CliServer {
    pub fn into_server(self, connection_config: crate::common::ConnectionConfig) -> Server {
        let send = match self.send {
            Some(cli_send) => Send::from(cli_send),
            None => Send::send(),
        };
        Server {
            connection_config,
            send,
        }
    }
}

impl CliCustomServer {
    pub fn into_server(self) -> Server {
        let url: crate::common::AvailableRpcServerUrl = match self.url {
            Some(url) => url,
            None => Input::new()
                .with_prompt("What is the RPC endpoint?")
                .interact_text()
                .unwrap(),
        };
        let send = match self.send {
            Some(cli_send) => Send::from(cli_send),
            None => Send::send(),
        };
        Server {
            connection_config: crate::common::ConnectionConfig::Custom { url: url.inner },
            send,
        }
    }
}

impl Server {
    pub async fn process(self) -> crate::CliResult {
        self.send.process(self.connection_config).await
    }
}

#[derive(Debug, Clone, clap::Clap)]
pub enum CliSend {
    /// Specify a transaction
    Transaction(super::super::super::super::CliTransaction),
}

#[derive(Debug, Clone)]
pub enum Send {
    Transaction(super::super::super::super::Transaction),
}

impl CliSend {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::Transaction(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("transaction".to_owned());
                args
            }
        }
    }
}

impl From<Send> for CliSend {
    fn from(send: Send) -> Self {
        match send {
            Send::Transaction(transaction) => Self::Transaction(transaction.into()),
        }
    }
}

impl From<CliSend> for Send {
    fn from(item: CliSend) -> Self {
        match item {
            CliSend::Transaction(cli_transaction) => {
                let transaction = super::super::super::super::Transaction::from(cli_transaction);
                Self::Transaction(transaction)
            }
        }
    }
}

impl Send {
    fn send() -> Self {
        Self::from(CliSend::Transaction(Default::default()))
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        match self {
            Send::Transaction(transaction) => transaction.process(network_connection_config).await,
        }
    }
}
