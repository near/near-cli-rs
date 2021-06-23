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
    pub send_to: Option<super::super::super::super::receiver::CliSendTo>,
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
    send_to: Option<super::super::super::super::receiver::CliSendTo>,
}

#[derive(Debug)]
pub struct Server {
    pub network_connection_config: Option<crate::common::ConnectionConfig>,
    pub send_to: super::super::super::super::receiver::SendTo,
}

impl CliServer {
    pub fn into_server(self, network_connection_config: crate::common::ConnectionConfig) -> Server {
        let send_to = match self.send_to {
            Some(cli_send_to) => super::super::super::super::receiver::SendTo::from(cli_send_to),
            None => super::super::super::super::receiver::SendTo::send_to(),
        };
        Server {
            network_connection_config: Some(network_connection_config),
            send_to,
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
        let send_to = match self.send_to {
            Some(cli_send_to) => super::super::super::super::receiver::SendTo::from(cli_send_to),
            None => super::super::super::super::receiver::SendTo::send_to(),
        };
        Server {
            network_connection_config: Some(crate::common::ConnectionConfig::Custom {
                url: url.inner,
            }),
            send_to,
        }
    }
}

impl Server {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.send_to
            .process(
                prepopulated_unsigned_transaction,
                self.network_connection_config,
            )
            .await
    }
}
