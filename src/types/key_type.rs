#[derive(Debug, Clone)]
pub struct KeyType(pub near_crypto::KeyType);

impl std::fmt::Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for KeyType {
    type Err = near_crypto::ParseKeyTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        near_crypto::KeyType::from_str(s).map(KeyType)
    }
}

impl interactive_clap::ToCli for KeyType {
    type CliVariant = KeyType;
}

pub fn near_key_type_to_mpc_domain_id(key_type: near_crypto::KeyType) -> u64 {
    match key_type {
        near_crypto::KeyType::SECP256K1 => 0u64,
        near_crypto::KeyType::ED25519 => 1u64,
    }
}

impl KeyType {
    pub fn to_mpc_domain_id(&self) -> u64 {
        near_key_type_to_mpc_domain_id(self.0)
    }
}
