use std::str::FromStr;

use dialoguer::Input;

/// предустановленный RPC-сервер
#[derive(Debug, Default, Clone, clap::Clap)]
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
#[derive(Debug, Default, Clone, clap::Clap)]
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

#[derive(Debug, Clone)]
pub struct Server {
    pub connection_config: Option<crate::common::ConnectionConfig>,
    pub send_from: SendFrom,
}

impl CliCustomServer {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .send_from
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
                    server.connection_config.unwrap().rpc_url().as_str(),
                )
                .unwrap(),
            ),
            send_from: Some(server.send_from.into()),
        }
    }
}

impl CliServer {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        self.send_from
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default()
    }
}

impl From<Server> for CliServer {
    fn from(server: Server) -> Self {
        Self {
            send_from: Some(server.send_from.into()),
        }
    }
}

impl CliServer {
    pub fn into_server(
        self,
        connection_config: crate::common::ConnectionConfig,
    ) -> color_eyre::eyre::Result<Server> {
        let send_from = match self.send_from {
            Some(cli_send_from) => SendFrom::from(cli_send_from, Some(connection_config.clone()))?,
            None => SendFrom::choose_send_from(Some(connection_config.clone()))?,
        };
        Ok(Server {
            connection_config: Some(connection_config),
            send_from,
        })
    }
}

impl CliCustomServer {
    pub fn into_server(self) -> color_eyre::eyre::Result<Server> {
        let url: crate::common::AvailableRpcServerUrl = match self.url {
            Some(url) => url,
            None => Input::new()
                .with_prompt("What is the RPC endpoint?")
                .interact_text()
                .unwrap(),
        };
        let connection_config = Some(crate::common::ConnectionConfig::Custom { url: url.inner });
        let send_from = match self.send_from {
            Some(cli_send_from) => SendFrom::from(cli_send_from, connection_config.clone()),
            None => SendFrom::choose_send_from(connection_config.clone()),
        }?;
        Ok(Server {
            connection_config,
            send_from,
        })
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

#[derive(Debug, Clone, clap::Clap)]
pub enum CliSendFrom {
    /// Specify a sender
    Account(super::super::super::super::sender::CliSender),
}

#[derive(Debug, Clone)]
pub enum SendFrom {
    Account(super::super::super::super::sender::Sender),
}

impl CliSendFrom {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::Account(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("account".to_owned());
                args
            }
        }
    }
}

impl From<SendFrom> for CliSendFrom {
    fn from(send_from: SendFrom) -> Self {
        match send_from {
            SendFrom::Account(sender) => Self::Account(sender.into()),
        }
    }
}

impl SendFrom {
    pub fn from(
        item: CliSendFrom,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliSendFrom::Account(cli_sender) => Ok(Self::Account(
                super::super::super::super::sender::Sender::from(
                    cli_sender,
                    connection_config.clone(),
                )?,
            )),
        }
    }
}

impl SendFrom {
    pub fn choose_send_from(
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self::from(
            CliSendFrom::Account(Default::default()),
            connection_config,
        )?)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self {
            SendFrom::Account(sender) => {
                sender
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}
