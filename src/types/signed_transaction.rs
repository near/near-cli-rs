use base64::Engine as _;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct SignedTransactionAsBase64 {
    #[serde(
        serialize_with = "serialize_as_base64",
        deserialize_with = "deserialize_from_base64"
    )]
    pub inner: near_kit::SignedTransactionV1,
}

fn serialize_as_base64<S>(
    tx: &near_kit::SignedTransactionV1,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&tx.to_base64())
}

fn deserialize_from_base64<'de, D>(
    deserializer: D,
) -> Result<near_kit::SignedTransactionV1, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    near_kit::SignedTransactionV1::from_base64(&s).map_err(serde::de::Error::custom)
}

impl From<SignedTransactionAsBase64> for near_kit::SignedTransactionV1 {
    fn from(transaction: SignedTransactionAsBase64) -> Self {
        transaction.inner
    }
}

impl std::str::FromStr for SignedTransactionAsBase64 {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(s)
            .map_err(|err| format!("base64 transaction sequence is invalid: {err}"))?;
        let inner = near_kit::SignedTransactionV1::from_bytes(&bytes)
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

impl From<near_kit::SignedTransactionV1> for SignedTransactionAsBase64 {
    fn from(value: near_kit::SignedTransactionV1) -> Self {
        Self { inner: value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transaction_and_key() -> (near_kit::Transaction, near_kit::SecretKey) {
        let secret_key = near_kit::SecretKey::generate_ed25519();
        let transaction = near_kit::Transaction::new(
            "alice.testnet".parse().unwrap(),
            secret_key.public_key(),
            7,
            "bob.testnet".parse().unwrap(),
            near_kit::CryptoHash::ZERO,
            Vec::new(),
        );
        (transaction, secret_key)
    }

    #[test]
    fn decodes_and_preserves_v0_wire_format() {
        let (transaction, secret_key) = transaction_and_key();
        let legacy_signed = transaction.sign(&secret_key);
        let encoded = legacy_signed.to_base64();
        let decoded: SignedTransactionAsBase64 = encoded.parse().unwrap();

        assert_eq!(decoded.inner.to_base64(), encoded);
    }

    #[test]
    fn round_trips_v1_wire_format() {
        let (transaction, secret_key) = transaction_and_key();
        let signed = transaction
            .into_gas_key_v1(3, near_kit::TransactionNonceMode::Strict)
            .sign(&secret_key);
        let encoded = SignedTransactionAsBase64::from(signed.clone()).to_string();
        let decoded: SignedTransactionAsBase64 = encoded.parse().unwrap();

        assert_eq!(decoded.inner, signed);
    }
}
