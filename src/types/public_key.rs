use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey(pub near_kit::PublicKey);

impl std::fmt::Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for PublicKey {
    type Err = color_eyre::eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let public_key = near_kit::PublicKey::from_str(s)
            .map_err(color_eyre::eyre::Report::msg)?;
        Ok(Self(public_key))
    }
}

impl From<PublicKey> for near_crypto::PublicKey {
    fn from(item: PublicKey) -> Self {
        // Both use identical "keytype:base58" format
        near_crypto::PublicKey::from_str(&item.0.to_string())
            .expect("near-kit and near-crypto use compatible public key formats")
    }
}

impl From<near_crypto::PublicKey> for PublicKey {
    fn from(item: near_crypto::PublicKey) -> Self {
        // Both use identical "keytype:base58" format
        Self(near_kit::PublicKey::from_str(&item.to_string())
            .expect("near-crypto and near-kit use compatible public key formats"))
    }
}

impl interactive_clap::ToCli for PublicKey {
    type CliVariant = PublicKey;
}
