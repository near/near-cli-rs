use dialoguer::Input;

/// предустановленный RPC-сервер
#[derive(Debug, Default, clap::Clap)]
pub struct CliServer {
    #[clap(subcommand)]
    send: Option<CliSend>,
}

/// данные для custom server
#[derive(Debug, Default, clap::Clap)]
pub struct CliCustomServer {
    #[clap(long)]
    pub url: Option<crate::common::AvailableRpcServerUrl>,
    #[clap(subcommand)]
    send: Option<CliSend>,
}

#[derive(Debug)]
pub struct Server {
    pub connection_config: crate::common::ConnectionConfig,
    send: Send,
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

#[derive(Debug, clap::Clap)]
pub enum CliSend {
    /// Specify a transaction
    Transaction(super::super::super::super::CliTransaction),
}

#[derive(Debug)]
pub enum Send {
    Transaction(super::super::super::super::Transaction),
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
