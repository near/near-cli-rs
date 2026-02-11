use std::str::FromStr;

/// Generate a key pair of private and public keys (use it anywhere you need
/// Ed25519 keys)
#[derive(Debug, Clone, clap::Parser)]
pub struct CliGenerateKeypair {
    #[arg(long)]
    pub master_seed_phrase: Option<String>,
    #[arg(long, default_value = "12")]
    pub new_master_seed_phrase_words_count: usize,
    #[arg(long, default_value = "m/44'/397'/0'")]
    pub seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[arg(long, default_value = "plaintext")]
    pub format: crate::common::OutputFormat,
}

impl Default for CliGenerateKeypair {
    fn default() -> Self {
        Self {
            master_seed_phrase: None,
            new_master_seed_phrase_words_count: 12,
            seed_phrase_hd_path: crate::types::slip10::BIP32Path::from_str("m/44'/397'/0'")
                .unwrap(),
            format: crate::common::OutputFormat::Json,
        }
    }
}
