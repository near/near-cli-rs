#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretKey(pub near_crypto::SecretKey);

impl std::fmt::Display for SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for SecretKey {
    type Err = near_crypto::ParseKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let private_key = near_crypto::SecretKey::from_str(s)?;
        // An ed25519 secret key carries its public half. near-crypto parses a
        // key whose halves do not match, then panics when signing with it.
        if let near_crypto::SecretKey::ED25519(key) = &private_key {
            ed25519_dalek::SigningKey::from_keypair_bytes(&key.0).map_err(|_| {
                near_crypto::ParseKeyError::InvalidData {
                    error_message: "the public key part does not match the secret key part"
                        .to_string(),
                }
            })?;
        }
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
    use std::str::FromStr;

    #[test]
    fn ed25519_key_with_mismatched_public_half_is_rejected() {
        let valid = near_crypto::SecretKey::from_seed(near_crypto::KeyType::ED25519, "test");
        assert!(SecretKey::from_str(&valid.to_string()).is_ok());

        let near_crypto::SecretKey::ED25519(mut key) = valid else {
            unreachable!()
        };
        key.0[63] ^= 1;
        let mismatched = near_crypto::SecretKey::ED25519(key).to_string();
        assert!(near_crypto::SecretKey::from_str(&mismatched).is_ok());
        assert!(SecretKey::from_str(&mismatched).is_err());
    }
}
