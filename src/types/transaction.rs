use near_primitives::borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone)]
pub struct TransactionAsBase64 {
    pub inner: near_primitives::transaction::Transaction,
}

impl From<TransactionAsBase64> for near_primitives::transaction::Transaction {
    fn from(transaction: TransactionAsBase64) -> Self {
        transaction.inner
    }
}

impl From<near_primitives::transaction::Transaction> for TransactionAsBase64 {
    fn from(value: near_primitives::transaction::Transaction) -> Self {
        Self { inner: value }
    }
}

impl interactive_clap::ToCli for TransactionAsBase64 {
    type CliVariant = TransactionAsBase64;
}

impl std::str::FromStr for TransactionAsBase64 {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: near_primitives::transaction::Transaction::try_from_slice(
                &near_primitives::serialize::from_base64(s)
                    .map_err(|err| format!("base64 transaction sequence is invalid: {}", err))?,
            )
            .map_err(|err| format!("transaction could not be parsed: {}", err))?,
        })
    }
}

impl std::fmt::Display for TransactionAsBase64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base64_unsigned_transaction = near_primitives::serialize::to_base64(
            self.inner
                .try_to_vec()
                .expect("Transaction is not expected to fail on serialization"),
        );
        write!(f, "{}", base64_unsigned_transaction)
    }
}
