#[derive(Debug, Default, Clone)]
pub struct CryptoHash(pub near_primitives::hash::CryptoHash);

impl From<CryptoHash> for near_primitives::hash::CryptoHash {
    fn from(item: CryptoHash) -> Self {
        item.0
    }
}

impl std::fmt::Display for CryptoHash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for CryptoHash {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let crypto_hash =
            near_primitives::hash::CryptoHash::from_str(s).map_err(|err| err.to_string())?;
        Ok(Self(crypto_hash))
    }
}

impl interactive_clap::ToCli for CryptoHash {
    type CliVariant = CryptoHash;
}
