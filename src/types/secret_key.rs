use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct SecretKey(pub near_kit::SecretKey);

impl std::fmt::Display for SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for SecretKey {
    type Err = color_eyre::eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let private_key = near_kit::SecretKey::from_str(s)
            .map_err(color_eyre::eyre::Report::msg)?;
        Ok(Self(private_key))
    }
}

impl From<SecretKey> for near_crypto::SecretKey {
    fn from(item: SecretKey) -> Self {
        // Both use identical "keytype:base58" format
        near_crypto::SecretKey::from_str(&item.0.to_string())
            .expect("near-kit and near-crypto use compatible secret key formats")
    }
}

impl interactive_clap::ToCli for SecretKey {
    type CliVariant = SecretKey;
}
