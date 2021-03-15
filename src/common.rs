use near_primitives::borsh::BorshDeserialize;
use std::convert::TryInto;

#[derive(Debug, Clone)]
pub struct TransactionAsBase64 {
    pub inner: near_primitives::transaction::Transaction,
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
        write!(f, "Transaction {}", self.inner.get_hash())
    }
}

#[derive(Debug, Clone)]
pub struct BlockHashAsBase58 {
    pub inner: near_primitives::hash::CryptoHash,
}

impl std::str::FromStr for BlockHashAsBase58 {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: near_primitives::serialize::from_base(s)
                .map_err(|err| format!("base block hash sequence is invalid: {}", err))?
                .as_slice()
                .try_into()
                .map_err(|err| format!("block hash could not be collected: {}", err))?,
        })
    }
}

impl std::fmt::Display for BlockHashAsBase58 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockHash {}", self.inner)
    }
}
