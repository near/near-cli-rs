use near_primitives::{borsh, borsh::BorshDeserialize};

#[derive(Debug, Clone)]
pub struct TransactionAsBase64 {
    pub inner: omni_transaction::near::NearTransaction,
}

impl From<TransactionAsBase64> for near_primitives::transaction::Transaction {
    fn from(transaction: TransactionAsBase64) -> Self {
        crate::types::omni_transaction_helpers::omni_transaction_to_near_primitives(&transaction.inner)
            .expect("Failed to convert omni-transaction to near-primitives")
    }
}

impl From<near_primitives::transaction::Transaction> for TransactionAsBase64 {
    fn from(value: near_primitives::transaction::Transaction) -> Self {
        let inner = crate::types::omni_transaction_helpers::near_primitives_transaction_to_omni(&value)
            .expect("Failed to convert near-primitives to omni-transaction");
        Self { inner }
    }
}

impl From<omni_transaction::near::NearTransaction> for TransactionAsBase64 {
    fn from(value: omni_transaction::near::NearTransaction) -> Self {
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
            inner: omni_transaction::near::NearTransaction::try_from_slice(
                &near_primitives::serialize::from_base64(s)
                    .map_err(|err| format!("base64 transaction sequence is invalid: {err}"))?,
            )
            .map_err(|err| format!("transaction could not be parsed: {err}"))?,
        })
    }
}

impl std::fmt::Display for TransactionAsBase64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base64_unsigned_transaction = near_primitives::serialize::to_base64(
            &borsh::to_vec(&self.inner)
                .expect("Transaction is not expected to fail on serialization"),
        );
        write!(f, "{base64_unsigned_transaction}")
    }
}
