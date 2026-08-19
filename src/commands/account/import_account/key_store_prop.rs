use serde_with::{DisplayFromStr, serde_as};

#[serde_as]
#[derive(Debug, Clone, serde::Serialize)]
pub struct RecoverableKey {
    #[serde_as(as = "DisplayFromStr")]
    pub seed_phrase_hd_path: near_slip10::BIP32Path,
    #[serde_as(as = "DisplayFromStr")]
    pub master_seed_phrase: bip39::Mnemonic,
    pub implicit_account_id: near_primitives::types::AccountId,
    #[serde(rename = "private_key")]
    pub secret_key: near_crypto::SecretKey,
    pub public_key: near_crypto::PublicKey,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PrimitiveKey {
    #[serde(rename = "private_key")]
    pub secret_key: near_crypto::SecretKey,
    pub public_key: near_crypto::PublicKey,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(untagged)]
pub enum KeyStorePropertyType {
    Recoverable(RecoverableKey),
    Primitive(PrimitiveKey),
    // Other(String),
}

impl KeyStorePropertyType {
    pub fn to_public_key_str(&self) -> String {
        match self {
            Self::Recoverable(prop) => {
                near_crypto::PublicKeyHandle::from(prop.public_key.clone()).to_string()
            }
            Self::Primitive(prop) => {
                near_crypto::PublicKeyHandle::from(prop.public_key.clone()).to_string()
            }
        }
    }
}
