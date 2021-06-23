use dialoguer::Input;

/// предустановленный RPC-сервер
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliServer {
    #[clap(subcommand)]
    pub send_from: Option<CliSendFrom>,
}

/// данные для custom server
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliCustomServer {
    #[clap(long)]
    pub url: Option<crate::common::AvailableRpcServerUrl>,
    #[clap(subcommand)]
    send_from: Option<CliSendFrom>,
}

#[derive(Debug)]
pub struct Server {
    pub connection_config: Option<crate::common::ConnectionConfig>,
    pub send_from: SendFrom,
}

impl CliServer {
    pub fn into_server(self, connection_config: crate::common::ConnectionConfig) -> Server {
        let send_from = match self.send_from {
            Some(cli_send_from) => SendFrom::from(cli_send_from),
            None => SendFrom::choose_send_from(),
        };
        Server {
            connection_config: Some(connection_config),
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
            connection_config: Some(crate::common::ConnectionConfig::Custom { url: url.inner }),
            send_from,
        }
    }
}

impl Server {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.send_from
            .process(prepopulated_unsigned_transaction, self.connection_config)
            .await
    }
}

#[derive(Debug, clap::Clap)]
pub enum CliSendFrom {
    /// Specify a sender
    Sender(crate::commands::transfer_command::sender::CliSender),
}

#[derive(Debug)]
pub enum SendFrom {
    Sender(crate::commands::transfer_command::sender::Sender),
}

impl From<CliSendFrom> for SendFrom {
    fn from(item: CliSendFrom) -> Self {
        match item {
            CliSendFrom::Sender(cli_sender) => Self::Sender(cli_sender.into()),
        }
    }
}

impl SendFrom {
    pub fn choose_send_from() -> Self {
        Self::from(CliSendFrom::Sender(Default::default()))
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self {
            SendFrom::Sender(sender) => {
                sender
                    .process(prepopulated_unsigned_transaction, connection_config)
                    .await
            }
        }
    }
}
