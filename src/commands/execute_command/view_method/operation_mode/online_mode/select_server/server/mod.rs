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
    pub send_to: Option<super::super::super::super::receiver::CliSendTo>,
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
    send_to: Option<super::super::super::super::receiver::CliSendTo>,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub network_connection_config: crate::common::ConnectionConfig,
    pub send_to: super::super::super::super::receiver::SendTo,
}

impl CliCustomServer {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .send_to
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
                    server.network_connection_config.rpc_url().as_str(),
                )
                .unwrap(),
            ),
            send_to: Some(server.send_to.into()),
        }
    }
}

impl CliServer {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        self.send_to
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default()
    }
}

impl From<Server> for CliServer {
    fn from(server: Server) -> Self {
        Self {
            send_to: Some(server.send_to.into()),
        }
    }
}

impl CliServer {
    pub fn into_server(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> color_eyre::eyre::Result<Server> {
        let send_to = match self.send_to {
            Some(cli_send_to) => super::super::super::super::receiver::SendTo::from(
                cli_send_to,
                network_connection_config.clone(),
            )?,
            None => super::super::super::super::receiver::SendTo::send_to(
                network_connection_config.clone(),
            )?,
        };
        Ok(Server {
            network_connection_config,
            send_to,
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
        let connection_config = crate::common::ConnectionConfig::Custom {
            url: url.inner.clone(),
        };
        let send_to = match self.send_to {
            Some(cli_send_to) => super::super::super::super::receiver::SendTo::from(
                cli_send_to,
                connection_config.clone(),
            )?,
            None => super::super::super::super::receiver::SendTo::send_to(connection_config)?,
        };
        Ok(Server {
            network_connection_config: crate::common::ConnectionConfig::Custom { url: url.inner },
            send_to,
        })
    }
}

impl Server {
    pub async fn process(self) -> crate::CliResult {
        self.send_to.process(self.network_connection_config).await
    }
}
