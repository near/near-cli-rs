#[derive(Clone, Debug)]
pub struct Transaction(pub near_primitives::transaction::Transaction);

impl From<Transaction> for near_primitives::transaction::Transaction {
    fn from(transaction: Transaction) -> Self {
        transaction.0
    }
}

impl interactive_clap::ToCli for Transaction {
    type CliVariant = Transaction;
}

impl std::str::FromStr for Transaction {
    type Err = color_eyre::eyre::ErrReport;

    fn from_str(transaction_str: &str) -> Result<Self, Self::Err> {
        let transaction: near_primitives::transaction::Transaction = serde_json::from_str(transaction_str)?;
        Ok(Self(transaction))
    }
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
