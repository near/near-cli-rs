use base64::{Engine as _, engine::general_purpose::STANDARD};
use borsh::BorshDeserialize;

#[derive(Debug, Clone)]
pub struct TransactionAsBase64 {
    pub inner: near_kit::Transaction,
}

impl From<TransactionAsBase64> for near_kit::Transaction {
    fn from(transaction: TransactionAsBase64) -> Self {
        transaction.inner
    }
}

impl From<near_kit::Transaction> for TransactionAsBase64 {
    fn from(value: near_kit::Transaction) -> Self {
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
            inner: near_kit::Transaction::try_from_slice(
                &STANDARD
                    .decode(s)
                    .map_err(|err| format!("base64 transaction sequence is invalid: {err}"))?,
            )
            .map_err(|err| format!("transaction could not be parsed: {err}"))?,
        })
    }
}

impl std::fmt::Display for TransactionAsBase64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base64_unsigned_transaction = STANDARD.encode(
            borsh::to_vec(&self.inner)
                .expect("Transaction is not expected to fail on serialization"),
        );
        write!(f, "{base64_unsigned_transaction}")
    }
}
