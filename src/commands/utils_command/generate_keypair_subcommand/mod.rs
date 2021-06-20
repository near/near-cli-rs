use std::str::FromStr;

/// Generate a key pair of secret and public keys (use it anywhere you need
/// Ed25519 keys)
#[derive(Debug, clap::Clap, Clone)]
pub struct CliGenerateKeypair {
    #[clap(long)]
    pub master_seed_phrase: Option<String>,
    #[clap(long, default_value = "12")]
    pub new_master_seed_phrase_words_count: usize,
    #[clap(long, default_value = "m/44'/397'/0'")]
    pub seed_phrase_hd_path: slip10::BIP32Path,
    #[clap(long, default_value = "plaintext")]
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
        let key_pair_properties = crate::common::generate_keypair(
            self.master_seed_phrase.as_deref(),
            self.new_master_seed_phrase_words_count,
            self.seed_phrase_hd_path,
        )
        .await?;

        match self.format {
            crate::common::OutputFormat::Plaintext => {
                println!(
                    "Master Seed Phrase: {}\nSeed Phrase HD Path: {}\nImplicit Account ID: {}\nPublic Key: {}\nSECRET KEYPAIR: {}",
                    key_pair_properties.master_seed_phrase,
                    key_pair_properties.seed_phrase_hd_path.to_string(),
                    key_pair_properties.implicit_account_id,
                    key_pair_properties.public_key_str,
                    key_pair_properties.secret_keypair_str,
                );
            }
            crate::common::OutputFormat::Json => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "master_seed_phrase": key_pair_properties.master_seed_phrase,
                        "seed_phrase_hd_path": key_pair_properties.seed_phrase_hd_path.to_string(),
                        "account_id": key_pair_properties.implicit_account_id,
                        "public_key": key_pair_properties.public_key_str,
                        "private_key": key_pair_properties.secret_keypair_str,
                    }))
                    .unwrap()
                );
            }
        };
        Ok(())
    }
}
