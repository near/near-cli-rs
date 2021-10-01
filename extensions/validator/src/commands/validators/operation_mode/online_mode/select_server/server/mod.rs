use dialoguer::Input;
use std::str::FromStr;

/// предустановленный RPC-сервер
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliServer {
    #[clap(subcommand)]
    pub epoch: Option<super::super::super::super::epoch::CliEpochCommand>,
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
    epoch: Option<super::super::super::super::epoch::CliEpochCommand>,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub connection_config: crate::common::ConnectionConfig,
    pub epoch: super::super::super::super::epoch::EpochCommand,
}

impl CliCustomServer {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .epoch
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
            epoch: Some(server.epoch.into()),
        }
    }
}

impl CliServer {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        self.epoch
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default()
    }
}

impl From<Server> for CliServer {
    fn from(server: Server) -> Self {
        Self {
            epoch: Some(server.epoch.into()),
        }
    }
}

impl CliServer {
    pub fn into_server(self, connection_config: crate::common::ConnectionConfig) -> Server {
        let send_to = match self.epoch {
            Some(cli_send_to) => cli_send_to.into(),
            None => super::super::super::super::epoch::EpochCommand::choose_command(),
        };
        Server {
            connection_config,
            epoch: send_to,
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
        let send_to = match self.epoch {
            Some(cli_send_to) => cli_send_to.into(),
            None => super::super::super::super::epoch::EpochCommand::choose_command(),
        };
        Server {
            connection_config: crate::common::ConnectionConfig::Custom { url: url.inner },
            epoch: send_to,
        }
    }
}

impl Server {
    pub async fn process(self) -> crate::CliResult {
        self.epoch.process(self.connection_config).await
    }
}
