use base64::Engine as _;
use borsh::BorshDeserialize;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct SignedTransactionAsBase64 {
    #[serde(
        serialize_with = "serialize_as_base64",
        deserialize_with = "deserialize_from_base64"
    )]
    pub inner: near_kit::SignedTransaction,
}

fn serialize_as_base64<S>(tx: &near_kit::SignedTransaction, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&tx.to_base64())
}

fn deserialize_from_base64<'de, D>(deserializer: D) -> Result<near_kit::SignedTransaction, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    near_kit::SignedTransaction::from_base64(&s).map_err(serde::de::Error::custom)
}

impl From<SignedTransactionAsBase64> for near_kit::SignedTransaction {
    fn from(transaction: SignedTransactionAsBase64) -> Self {
        transaction.inner
    }
}

impl std::str::FromStr for SignedTransactionAsBase64 {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = base64::engine::general_purpose::STANDARD.decode(s)
            .map_err(|err| format!("base64 transaction sequence is invalid: {err}"))?;
        let inner = near_kit::SignedTransaction::deserialize(&mut &bytes[..])
            .map_err(|err| format!("transaction could not be parsed: {err}"))?;
        Ok(Self { inner })
    }
}

impl std::fmt::Display for SignedTransactionAsBase64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.to_base64())
    }
}

impl interactive_clap::ToCli for SignedTransactionAsBase64 {
    type CliVariant = SignedTransactionAsBase64;
}

impl From<near_kit::SignedTransaction> for SignedTransactionAsBase64 {
    fn from(value: near_kit::SignedTransaction) -> Self {
        Self { inner: value }
    }
}
