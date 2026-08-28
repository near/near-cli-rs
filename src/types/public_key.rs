#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey(pub near_kit::PublicKey);

/// Parse a user-provided public key.
///
/// `ml-dsa-65-hash:...` strings (the on-chain handle of an ML-DSA-65 access
/// key, which `list-keys` and view RPCs print) are rejected by near-kit itself
/// with [`near_kit::error::ParseKeyError::MlDsa65HashHandle`], which already
/// tells the user to supply the full `ml-dsa-65:` key instead.
pub fn parse_public_key(s: &str) -> color_eyre::eyre::Result<near_kit::PublicKey> {
    s.parse::<near_kit::PublicKey>()
        .map_err(color_eyre::eyre::Report::msg)
}

impl std::fmt::Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for PublicKey {
    type Err = color_eyre::eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(parse_public_key(s)?))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ed25519_public_key() {
        let public_key: PublicKey = "ed25519:5387nnYC7uiWrrPevw7FAopaL8hfr6dZVJqpg6HPrPKr"
            .parse()
            .unwrap();
        assert_eq!(
            public_key.to_string(),
            "ed25519:5387nnYC7uiWrrPevw7FAopaL8hfr6dZVJqpg6HPrPKr"
        );
    }

    #[test]
    fn rejects_ml_dsa_65_hash_handle() {
        // near-kit refuses to parse the on-chain hash handle as a public key;
        // this pins that its error (with the "supply the full key" guidance)
        // surfaces to the user unchanged.
        let err = "ml-dsa-65-hash:11111111111111111111111111111111"
            .parse::<PublicKey>()
            .unwrap_err();
        let message = err.to_string();
        assert!(
            message.contains("handle (hash) of an ML-DSA-65 key, not a public key")
                && message.contains("supply the full 'ml-dsa-65:' key"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_garbage() {
        assert!("not-a-key".parse::<PublicKey>().is_err());
    }
}
