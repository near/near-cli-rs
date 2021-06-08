use std::str::FromStr;

fn bip32path_to_string(bip32path: &slip10::BIP32Path) -> String {
    const HARDEND: u32 = 1 << 31;

    format!(
        "m/{}",
        (0..bip32path.depth())
            .map(|index| {
                let value = *bip32path.index(index).unwrap();
                if value < HARDEND {
                    value.to_string()
                } else {
                    format!("{}'", value - HARDEND)
                }
            })
            .collect::<Vec<String>>()
            .join("/")
    )
}

/// Generate a key pair of secret and public keys (use it anywhere you need
/// Ed25519 keys)
#[derive(Debug, clap::Clap, Clone)]
pub struct CliGenerateKeypair {
    pub master_seed_phrase: Option<String>,
    pub new_master_seed_phrase_words_count: usize,
    pub seed_phrase_hd_path: slip10::BIP32Path,
    pub format: crate::common::OutputFormat,
}

impl Default for CliGenerateKeypair {
    fn default() -> Self {
        Self {
            master_seed_phrase: None,
            new_master_seed_phrase_words_count: 12,
            seed_phrase_hd_path: slip10::BIP32Path::from_str("m/44'/397'/0'").unwrap(),
            format: crate::common::OutputFormat::Json,
        }
    }
}

pub struct KeyPairProperties {
    master_seed_phrase: String,
    implicit_account_id: String,
    pub public_key_str: String,
    pub secret_keypair_str: String,
}

impl CliGenerateKeypair {
    pub async fn generate_keypair(self) -> color_eyre::eyre::Result<KeyPairProperties> {
        let (master_seed_phrase, master_seed) =
            if let Some(ref master_seed_phrase) = self.master_seed_phrase {
                (
                    master_seed_phrase.clone(),
                    bip39::Mnemonic::parse(master_seed_phrase)?.to_seed(""),
                )
            } else {
                let mnemonic = bip39::Mnemonic::generate(self.new_master_seed_phrase_words_count)?;
                let mut master_seed_phrase = String::new();
                for (index, word) in mnemonic.word_iter().enumerate() {
                    if index != 0 {
                        master_seed_phrase.push(' ');
                    }
                    master_seed_phrase.push_str(word);
                }
                (master_seed_phrase, mnemonic.to_seed(""))
            };

        let derived_private_key = slip10::derive_key_from_path(
            &master_seed,
            slip10::Curve::Ed25519,
            &self.seed_phrase_hd_path,
        )
        .map_err(|err| {
            color_eyre::Report::msg(format!(
                "Failed to derive a key from the master key: {}",
                err
            ))
        })?;

        let secret_keypair = {
            let secret = ed25519_dalek::SecretKey::from_bytes(&derived_private_key.key)?;
            let public = ed25519_dalek::PublicKey::from(&secret);
            ed25519_dalek::Keypair { secret, public }
        };

        let implicit_account_id = hex::encode(&secret_keypair.public);
        let public_key_str = format!(
            "ed25519:{}",
            bs58::encode(&secret_keypair.public).into_string()
        );
        let secret_keypair_str = format!(
            "ed25519:{}",
            bs58::encode(secret_keypair.to_bytes()).into_string()
        );
        let key_pair_properties: KeyPairProperties = KeyPairProperties {
            master_seed_phrase,
            implicit_account_id,
            public_key_str,
            secret_keypair_str,
        };
        Ok(key_pair_properties)
    }

    pub async fn process(self) -> crate::CliResult {
        let self_clone = self.clone();

        let seed_phrase_hd_path = bip32path_to_string(&self_clone.seed_phrase_hd_path);
        let key_pair_properties = self.generate_keypair().await?;

        match self_clone.format {
            crate::common::OutputFormat::Plaintext => {
                println!(
                    "Master Seed Phrase: {}\nSeed Phrase HD Path: {}\nImplicit Account ID: {}\nPublic Key: {}\nSECRET KEYPAIR: {}",
                    key_pair_properties.master_seed_phrase,
                    // bip32path_to_string(&self.seed_phrase_hd_path.clone()),
                    seed_phrase_hd_path,
                    key_pair_properties.implicit_account_id,
                    key_pair_properties.public_key_str,
                    key_pair_properties.secret_keypair_str,
                );
            }
            crate::common::OutputFormat::Json => {
                println!(
                    "{:#?}",
                    serde_json::json!({
                        "master_seed_phrase": key_pair_properties.master_seed_phrase,
                        // "seed_phrase_hd_path": bip32path_to_string(&self.seed_phrase_hd_path.clone()),
                        "seed_phrase_hd_path": seed_phrase_hd_path,
                        "account_id": key_pair_properties.implicit_account_id,
                        "public_key": key_pair_properties.public_key_str,
                        "private_key": key_pair_properties.secret_keypair_str,
                    })
                );
            }
        };
        Ok(())
    }
}
