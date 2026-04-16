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

impl From<PublicKey> for near_kit::PublicKey {
    fn from(item: PublicKey) -> Self {
        item.0
    }
}

impl From<near_kit::PublicKey> for PublicKey {
    fn from(item: near_kit::PublicKey) -> Self {
        Self(item)
    }
}

impl interactive_clap::ToCli for PublicKey {
    type CliVariant = PublicKey;
}
