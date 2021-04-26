use dialoguer::Input;


/// предустановленный RPC-сервер
#[derive(Debug, Default, clap::Clap)]
pub struct CliServer {
    #[clap(subcommand)]
    pub send_from: Option<CliSendFrom>,
}

/// данные для custom server
#[derive(Debug, Default, clap::Clap)]
pub struct CliCustomServer {
    #[clap(long)]
    pub url: Option<crate::common::AvailableRpcServerUrl>,
    #[clap(subcommand)]
    send_from: Option<CliSendFrom>,
}

#[derive(Debug)]
pub struct Server {
    pub url: Option<url::Url>,
    pub send_from: SendFrom,
}

impl CliServer {
    pub fn into_server(self, url: url::Url) -> Server {
        let send_from = match self.send_from {
            Some(cli_send_from) => SendFrom::from(cli_send_from),
            None => SendFrom::choose_send_from(),
        };
        Server {
            url: Some(url),
            send_from,
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
        let send_from = match self.send_from {
            Some(cli_send_from) => SendFrom::from(cli_send_from),
            None => SendFrom::choose_send_from(),
        };
        Server {
            url: Some(url.inner),
            send_from,
        }
    }
}

impl Server {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        let selected_server_url = self.url.clone();
        self.send_from
            .process(prepopulated_unsigned_transaction, selected_server_url)
            .await
    }
}

#[derive(Debug, clap::Clap)]
pub enum CliSendFrom {
    /// Specify a sender
    Account(crate::commands::add_command::access_key::sender::CliSender),
}

#[derive(Debug)]
pub enum SendFrom {
    Account(crate::commands::add_command::access_key::sender::Sender),
}

impl From<CliSendFrom> for SendFrom {
    fn from(item: CliSendFrom) -> Self {
        match item {
            CliSendFrom::Account(cli_sender) => {
                Self::Account(cli_sender.into())
            }
        }
    }
}

impl SendFrom {
    pub fn choose_send_from() -> Self {
        Self::from(CliSendFrom::Account(Default::default()))
    }
    
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        match self {
            SendFrom::Account(sender) => {
                sender
                    .process(prepopulated_unsigned_transaction, selected_server_url)
                    .await
            }
        }
    }
}
