use dialoguer::Input;

/// предустановленный RPC-сервер
#[derive(Debug, Default, clap::Clap)]
pub struct CliServer {
    #[clap(subcommand)]
    pub send_to: Option<super::super::super::super::account::CliSendTo>,
}

/// данные для custom server
#[derive(Debug, Default, clap::Clap)]
pub struct CliCustomServer {
    #[clap(long)]
    pub url: Option<crate::common::AvailableRpcServerUrl>,
    #[clap(subcommand)]
    send_to: Option<super::super::super::super::account::CliSendTo>,
}

#[derive(Debug)]
pub struct Server {
    pub connection_config: super::ConnectionConfig,
    pub send_to: super::super::super::super::account::SendTo,
}

impl CliServer {
    pub fn into_server(self, connection_config: super::ConnectionConfig) -> Server {
        let send_to = match self.send_to {
            Some(cli_send_to) => super::super::super::super::account::SendTo::from(cli_send_to),
            None => super::super::super::super::account::SendTo::send_to(),
        };
        Server {
            connection_config,
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
            Some(cli_send_to) => super::super::super::super::account::SendTo::from(cli_send_to),
            None => super::super::super::super::account::SendTo::send_to(),
        };
        Server {
            connection_config: super::ConnectionConfig::Custom { url: url.inner },
            send_to,
        }
    }
}

impl Server {
    pub async fn process(self) -> crate::CliResult {
        self.send_to.process(self.connection_config).await
    }
}
