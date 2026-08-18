#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey(pub near_kit::PublicKey);

/// Parse a user-provided public key.
///
/// Rejects `ml-dsa-65-hash:...` strings: that is the on-chain handle of an
/// ML-DSA-65 access key (what `list-keys` and view RPCs print), not a public
/// key. A handle cannot be put into a transaction or an action, so accepting
/// it here would only fail later (at signing/serialization time) with a much
/// less helpful error.
pub fn parse_public_key(s: &str) -> color_eyre::eyre::Result<near_kit::PublicKey> {
    let public_key = s
        .parse::<near_kit::PublicKey>()
        .map_err(color_eyre::eyre::Report::msg)?;
    if public_key.is_ml_dsa65_hash() {
        return Err(color_eyre::eyre::eyre!(
            "`{public_key}` is the on-chain handle (hash) of an ML-DSA-65 key, not a public key. \
             Please provide the full `ml-dsa-65:...` public key instead."
        ));
    }
    Ok(public_key)
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
        let err = "ml-dsa-65-hash:11111111111111111111111111111111"
            .parse::<PublicKey>()
            .unwrap_err();
        assert!(
            err.to_string()
                .contains("handle (hash) of an ML-DSA-65 key, not a public key"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_garbage() {
        assert!("not-a-key".parse::<PublicKey>().is_err());
    }
}
