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
    pub transaction_status: Option<super::super::super::super::transaction::CliTransaction>,
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
    transaction_status: Option<super::super::super::super::transaction::CliTransaction>,
}

#[derive(Debug)]
pub struct Server {
    pub connection_config: crate::common::ConnectionConfig,
    pub transaction_status: super::super::super::super::transaction::Transaction,
}

impl CliServer {
    pub fn into_server(self, connection_config: crate::common::ConnectionConfig) -> Server {
        let transaction_status = match self.transaction_status {
            Some(cli_transaction_status) => {
                super::super::super::super::transaction::Transaction::from(cli_transaction_status)
            }
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
            Some(cli_transaction_status) => {
                super::super::super::super::transaction::Transaction::from(cli_transaction_status)
            }
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
