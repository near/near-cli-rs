use dialoguer::Input;


/// предустановленный RPC-сервер
#[derive(Debug, Default, clap::Clap)]
pub struct CliServer {
    #[clap(subcommand)]
    pub transaction_status: Option<super::super::super::super::CliTransactionStatus>,
}

/// данные для custom server
#[derive(Debug, Default, clap::Clap)]
pub struct CliCustomServer {
    #[clap(long)]
    pub url: Option<crate::common::AvailableRpcServerUrl>,
    #[clap(subcommand)]
    transaction_status: Option<super::super::super::super::CliTransactionStatus>,
}

#[derive(Debug)]
pub struct Server {
    pub url: url::Url,
    pub transaction_status: super::super::super::super::TransactionStatus,
}

impl CliServer {
    pub fn into_server(self, url: url::Url) -> Server {
        let transaction_status = match self.transaction_status {
            Some(cli_transaction_status) => super::super::super::super::TransactionStatus::from(cli_transaction_status),
            None => super::super::super::super::TransactionStatus::choose_transaction_status(),
        };
        Server {
            url,
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
            Some(cli_transaction_status) => super::super::super::super::TransactionStatus::from(cli_transaction_status),
            None => super::super::super::super::TransactionStatus::choose_transaction_status(),
        };
        Server {
            url: url.inner,
            transaction_status,
        }
    }
}

impl Server {
    pub async fn process(
        self,
    ) -> crate::CliResult {
        let selected_server_url = self.url.clone();
        self.transaction_status
            .process(selected_server_url)
            .await
    }
}
