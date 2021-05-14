use dialoguer::Input;

pub mod operation_mode;
mod sender;

#[derive(Debug, clap::Clap)]
pub enum CliTransactionStatus {
    /// Specify a transaction
    TransactionHash(CliTransactionType),
}

#[derive(Debug)]
pub enum TransactionStatus {
    TransactionHash(TransactionType),
}

impl From<CliTransactionStatus> for TransactionStatus {
    fn from(item: CliTransactionStatus) -> Self {
        match item {
            CliTransactionStatus::TransactionHash(cli_transaction_type) => {
                TransactionStatus::TransactionHash(cli_transaction_type.into())
            }
        }
    }
}

impl TransactionStatus {
    pub fn choose_transaction_status() -> Self {
        Self::from(CliTransactionStatus::TransactionHash(Default::default()))
    }

    pub async fn process(self, selected_server_url: url::Url) -> crate::CliResult {
        match self {
            TransactionStatus::TransactionHash(transaction_type) => {
                transaction_type.process(selected_server_url).await
            }
        }
    }
}

/// Specify the transaction to be view
#[derive(Debug, Default, clap::Clap)]
pub struct CliTransactionType {
    pub transaction_hash: Option<String>,
    #[clap(subcommand)]
    send_from: Option<self::sender::CliSendFrom>,
}

#[derive(Debug)]
pub struct TransactionType {
    pub transaction_hash: String,
    send_from: self::sender::SendFrom,
}

impl From<CliTransactionType> for TransactionType {
    fn from(item: CliTransactionType) -> Self {
        let transaction_hash = match item.transaction_hash {
            Some(cli_transaction_hash) => cli_transaction_hash,
            None => TransactionType::input_transaction_hash(),
        };
        let send_from = match item.send_from {
            Some(cli_send_from) => self::sender::SendFrom::from(cli_send_from),
            None => self::sender::SendFrom::send_from(),
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

    pub async fn process(self, selected_server_url: url::Url) -> crate::CliResult {
        self.send_from
            .process(selected_server_url, self.transaction_hash)
            .await
    }
}
