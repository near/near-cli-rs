#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct PublicKey(pub near_crypto::PublicKey);

impl std::fmt::Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for PublicKey {
    type Err = near_crypto::ParseKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let public_key = near_crypto::PublicKey::from_str(s)?;
        Ok(Self(public_key))
    }
}

impl From<PublicKey> for near_crypto::PublicKey {
    fn from(item: PublicKey) -> Self {
        item.0
    }
}

impl From<near_crypto::PublicKey> for PublicKey {
    fn from(item: near_crypto::PublicKey) -> Self {
        Self(item)
    }
}

impl interactive_clap::ToCli for PublicKey {
    type CliVariant = PublicKey;
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct PublicKeyOrKeyHandle(pub near_crypto::PublicKeyHandle);

impl std::fmt::Display for PublicKeyOrKeyHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for PublicKeyOrKeyHandle {
    type Err = near_crypto::ParseKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(public_key) = near_crypto::PublicKey::from_str(s) {
            return Ok(Self((&public_key).into()));
        }

        near_crypto::PublicKeyHandle::from_str(s).map(Self)
    }
}

impl interactive_clap::ToCli for PublicKeyOrKeyHandle {
    type CliVariant = PublicKeyOrKeyHandle;
}

#[cfg(test)]
mod tests {
    use super::PublicKeyOrKeyHandle;

    #[test]
    fn ml_dsa_full_public_key_and_handle_select_the_same_key() {
        let public_key =
            near_crypto::SecretKey::from_seed(near_crypto::KeyType::MLDSA65, "test").public_key();
        let expected_handle = near_crypto::PublicKeyHandle::from(&public_key);

        assert_eq!(
            public_key
                .to_string()
                .parse::<PublicKeyOrKeyHandle>()
                .unwrap()
                .0,
            expected_handle
        );
        assert_eq!(
            expected_handle
                .to_string()
                .parse::<PublicKeyOrKeyHandle>()
                .unwrap()
                .0,
            expected_handle
        );
    }
}
