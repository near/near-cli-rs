use dialoguer::Input;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliTransaction {
    /// Specify a transaction
    TransactionHash(CliTransactionType),
}

#[derive(Debug, Clone)]
pub enum Transaction {
    TransactionHash(TransactionType),
}

impl CliTransaction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::TransactionHash(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("transaction-hash".to_owned());
                args
            }
        }
    }
}

impl From<Transaction> for CliTransaction {
    fn from(transaction: Transaction) -> Self {
        match transaction {
            Transaction::TransactionHash(transaction_type) => {
                Self::TransactionHash(transaction_type.into())
            }
        }
    }
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
#[derive(Debug, Default, Clone, clap::Clap)]
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

#[derive(Debug, Clone)]
pub struct TransactionType {
    pub transaction_hash: String,
    send_from: super::signer::SendFrom,
}

impl CliTransactionType {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .send_from
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(transaction_hash) = &self.transaction_hash {
            args.push_front(transaction_hash.to_string());
        };
        args
    }
}

impl From<TransactionType> for CliTransactionType {
    fn from(transaction_type: TransactionType) -> Self {
        Self {
            transaction_hash: Some(transaction_type.transaction_hash),
            send_from: Some(transaction_type.send_from.into()),
        }
    }
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
