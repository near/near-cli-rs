use std::str::FromStr;

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
}

impl RecoverableKey {
    pub fn generate_keypair() -> color_eyre::eyre::Result<Self> {
        // NOTE: `.unwrap()` is fine as this is default near web-wallets path and it is hardened
        let seed_phrase_hd_path = near_slip10::BIP32Path::from_str("m/44'/397'/0'").unwrap();
        // TODO: can change to 24 when support for ML-DSA-65 derivation lands
        let master_seed_phrase = bip39::Mnemonic::generate(12)?;

        Self::derive(master_seed_phrase, seed_phrase_hd_path)
    }

    pub fn derive(
        master_seed_phrase: bip39::Mnemonic,
        seed_phrase_hd_path: near_slip10::BIP32Path,
    ) -> color_eyre::eyre::Result<Self> {
        let derived_private_key = near_slip10::derive_key_from_path(
            &master_seed_phrase.to_seed(""),
            near_slip10::Curve::Ed25519,
            &seed_phrase_hd_path,
        )
        .map_err(|err| {
            color_eyre::Report::msg(format!("Failed to derive a key from the master key: {err}"))
        })?;

        let signing_key = ed25519_dalek::SigningKey::from_bytes(&derived_private_key.key);
        let secret_key = near_crypto::SecretKey::ED25519(near_crypto::ED25519SecretKey(
            signing_key.to_keypair_bytes(),
        ));
        let public_key = secret_key.public_key();

        let implicit_account_id =
            near_primitives::types::AccountId::try_from(hex::encode(public_key.key_data()))
                .map_err(|err| {
                    color_eyre::Report::msg(format!(
                        "Failed to parse implicit Account ID from derived public key: {err}"
                    ))
                })?;

        Ok(Self {
            seed_phrase_hd_path,
            master_seed_phrase,
            implicit_account_id,
            secret_key,
            public_key,
        })
    }
}

impl KeyStorePropertyType {
    pub fn public_key(&self) -> &near_crypto::PublicKey {
        match self {
            KeyStorePropertyType::Recoverable(prop) => &prop.public_key,
            KeyStorePropertyType::Primitive(prop) => &prop.public_key,
        }
    }

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
