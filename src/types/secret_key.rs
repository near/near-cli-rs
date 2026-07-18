#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretKey(pub near_crypto::SecretKey);

impl std::fmt::Display for SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl std::str::FromStr for SecretKey {
    type Err = near_crypto::ParseKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let private_key = near_crypto::SecretKey::from_str(s)?;
        Ok(Self(private_key))
    }
}

impl From<SecretKey> for near_crypto::SecretKey {
    fn from(item: SecretKey) -> Self {
        item.0
    }
}

impl interactive_clap::ToCli for SecretKey {
    type CliVariant = SecretKey;
}

#[cfg(test)]
mod tests {
    use super::SecretKey;

    #[test]
    fn display_redacts_the_secret_key() {
        let secret_key: SecretKey = "ed25519:3D4YudUahN1nawWogh8pAKSj92sUNMdbZGjn7kERKzYoTy8tnFQuwoGUC51DowKqorvkr2pytJSnwuSbsNVfqygr"
            .parse()
            .unwrap();

        assert_eq!(secret_key.to_string(), "[REDACTED]");
    }
}
