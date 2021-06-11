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

impl CliGenerateKeypair {
    pub async fn process(self) -> crate::CliResult {
        let seed_phrase_hd_path = bip32path_to_string(&self.seed_phrase_hd_path);

        let key_pair_properties = crate::common::generate_keypair(
            self.master_seed_phrase.clone(),
            self.new_master_seed_phrase_words_count.clone(),
            self.seed_phrase_hd_path.clone(),
        )
        .await?;

        match self.format {
            crate::common::OutputFormat::Plaintext => {
                println!(
                    "Master Seed Phrase: {}\nSeed Phrase HD Path: {}\nImplicit Account ID: {}\nPublic Key: {}\nSECRET KEYPAIR: {}",
                    key_pair_properties.master_seed_phrase,
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
