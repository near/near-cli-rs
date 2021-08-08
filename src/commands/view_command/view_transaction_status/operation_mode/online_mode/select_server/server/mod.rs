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
    pub transaction_status: Option<super::super::super::super::transaction::CliTransaction>,
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
    transaction_status: Option<super::super::super::super::transaction::CliTransaction>,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub connection_config: crate::common::ConnectionConfig,
    pub transaction_status: super::super::super::super::transaction::Transaction,
}

impl CliCustomServer {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .transaction_status
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
            transaction_status: Some(server.transaction_status.into()),
        }
    }
}

impl CliServer {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        self.transaction_status
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default()
    }
}

impl From<Server> for CliServer {
    fn from(server: Server) -> Self {
        Self {
            transaction_status: Some(server.transaction_status.into()),
        }
    }
}

impl CliServer {
    pub fn into_server(self, connection_config: crate::common::ConnectionConfig) -> Server {
        let transaction_status = match self.transaction_status {
            Some(cli_transaction_status) => cli_transaction_status.into(),
            None => super::super::super::super::transaction::Transaction::transaction(),
        };
        Server {
            connection_config,
            transaction_status,
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
        let transaction_status = match self.transaction_status {
            Some(cli_transaction_status) => cli_transaction_status.into(),
            None => super::super::super::super::transaction::Transaction::transaction(),
        };
        Server {
            connection_config: crate::common::ConnectionConfig::Custom { url: url.inner },
            transaction_status,
        }
    }
}

impl Server {
    pub async fn process(self) -> crate::CliResult {
        self.transaction_status
            .process(self.connection_config)
            .await
    }
}
