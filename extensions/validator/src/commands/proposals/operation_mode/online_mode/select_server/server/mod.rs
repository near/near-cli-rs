use dialoguer::Input;
use std::str::FromStr;

use crate::common::display_proposals_info;
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
}

#[derive(Debug, Clone)]
pub struct Server {
    pub connection_config: crate::common::ConnectionConfig,
}

impl CliCustomServer {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = std::collections::VecDeque::new();
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
        }
    }
}

// impl CliServer {
//     pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
//         let mut args = std::collections::VecDeque::new();
//         /////////////////////////
//         if let Some(server) = &self.server {
//             args.push_front(server.to_string());
//         }
//         args
//         //////////////////////
//         self.send_to
//             .as_ref()
//             .map(|subcommand| subcommand.to_cli_args())
//             .unwrap_or_default()
//     }
// }

// impl From<Server> for CliServer {
//     fn from(server: Server) -> Self {
//         Self {}
//     }
// }

// impl CliServer {
//     pub fn into_server(self, connection_config: crate::common::ConnectionConfig) -> Server {
//         Server { connection_config }
//     }
// }

impl CliCustomServer {
    pub fn into_server(self) -> Server {
        let url: crate::common::AvailableRpcServerUrl = match self.url {
            Some(url) => url,
            None => Input::new()
                .with_prompt("What is the RPC endpoint?")
                .interact_text()
                .unwrap(),
        };
        Server {
            connection_config: crate::common::ConnectionConfig::Custom { url: url.inner },
        }
    }
}

impl Server {
    pub async fn process(self) -> crate::CliResult {
        display_proposals_info(
            &self.connection_config,
        )
        .await?;
        Ok(())
    }
}
