#[derive(Clone, Debug)]
pub struct SignedTransaction(pub near_primitives::transaction::SignedTransaction);

impl From<SignedTransaction> for near_primitives::transaction::SignedTransaction {
    fn from(transaction: SignedTransaction) -> Self {
        transaction.0
    }
}

impl interactive_clap::ToCli for SignedTransaction {
    type CliVariant = SignedTransaction;
}

impl std::str::FromStr for SignedTransaction {
    type Err = color_eyre::eyre::ErrReport;

    fn from_str(transaction_str: &str) -> Result<Self, Self::Err> {
        let transaction: near_primitives::transaction::SignedTransaction =
            serde_json::from_str(transaction_str)?;
        Ok(Self(transaction))
    }
}

impl std::fmt::Display for SignedTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
