use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Signature(pub near_kit::Signature);

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for Signature {
    type Err = color_eyre::eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let signature = near_kit::Signature::from_str(s)
            .map_err(color_eyre::eyre::Report::msg)?;
        Ok(Self(signature))
    }
}

impl From<Signature> for near_kit::Signature {
    fn from(item: Signature) -> Self {
        // Both use identical "keytype:base58" format
        near_kit::Signature::from_str(&item.0.to_string())
            .expect("near-kit and near-crypto use compatible signature formats")
    }
}
