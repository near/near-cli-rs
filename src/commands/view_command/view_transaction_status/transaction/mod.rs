use dialoguer::Input;

#[derive(Debug, clap::Clap)]
pub enum CliTransaction {
    /// Specify a transaction
    TransactionHash(CliTransactionType),
}

#[derive(Debug)]
pub enum Transaction {
    TransactionHash(TransactionType),
}

impl From<CliTransaction> for Transaction {
    fn from(item: CliTransaction) -> Self {
        match item {
            CliTransaction::TransactionHash(cli_transaction_type) => {
                Transaction::TransactionHash(cli_transaction_type.into())
            }
        }
    }
}

impl Transaction {
    pub fn transaction() -> Self {
        Self::from(CliTransaction::TransactionHash(Default::default()))
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        match self {
            Transaction::TransactionHash(transaction_type) => {
                transaction_type.process(network_connection_config).await
            }
        }
    }
}

/// Specify the transaction to be view
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliTransactionType {
    pub transaction_hash: Option<String>,
    #[clap(subcommand)]
    send_from: Option<super::signer::CliSendFrom>,
}

#[derive(Debug)]
pub struct TransactionType {
    pub transaction_hash: String,
    send_from: super::signer::SendFrom,
}

impl From<CliTransactionType> for TransactionType {
    fn from(item: CliTransactionType) -> Self {
        let transaction_hash = match item.transaction_hash {
            Some(cli_transaction_hash) => cli_transaction_hash,
            None => TransactionType::input_transaction_hash(),
        };
        let send_from = match item.send_from {
            Some(cli_send_from) => super::signer::SendFrom::from(cli_send_from),
            None => super::signer::SendFrom::send_from(),
        };
        Self {
            transaction_hash,
            send_from,
        }
    }
}

impl TransactionType {
    fn input_transaction_hash() -> String {
        println!();
        Input::new()
            .with_prompt("Enter the hash of the transaction you need to view")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        self.send_from
            .process(network_connection_config, self.transaction_hash)
            .await
    }
}
