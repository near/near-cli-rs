#[derive(Debug, Copy, Clone)]
pub struct CryptoHash(pub near_kit::CryptoHash);

impl From<CryptoHash> for near_primitives::hash::CryptoHash {
    fn from(item: CryptoHash) -> Self {
        near_primitives::hash::CryptoHash(*item.0.as_bytes())
    }
}

impl From<near_primitives::hash::CryptoHash> for CryptoHash {
    fn from(item: near_primitives::hash::CryptoHash) -> Self {
        Self(near_kit::CryptoHash::from_bytes(item.0))
    }
}

impl std::fmt::Display for CryptoHash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for CryptoHash {
    type Err = color_eyre::eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let crypto_hash = near_kit::CryptoHash::from_str(s)
            .map_err(color_eyre::eyre::Report::msg)?;
        Ok(Self(crypto_hash))
    }
}

impl interactive_clap::ToCli for CryptoHash {
    type CliVariant = CryptoHash;
}
