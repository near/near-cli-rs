use std::str::FromStr;

pub mod near_ledger;

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
pub struct CliLedgerPublicKey {
    #[clap(long, default_value = "44'/397'/0'/0'/1'")]
    pub seed_phrase_hd_path: slip10::BIP32Path,
    #[clap(long, default_value = "plaintext")]
    pub format: crate::common::OutputFormat,
}

impl Default for CliLedgerPublicKey {
    fn default() -> Self {
        Self {
            seed_phrase_hd_path: slip10::BIP32Path::from_str("44'/397'/0'/0'/1'").unwrap(),
            format: crate::common::OutputFormat::Json,
        }
    }
}

impl CliLedgerPublicKey {
    pub async fn process(self) -> crate::CliResult {
        // Get public key from ledger
        // let key_pair_properties = crate::common::generate_keypair(
        //     self.master_seed_phrase.as_deref(),
        //     self.new_master_seed_phrase_words_count,
        //     self.seed_phrase_hd_path,
        // )
        // .await?;

        near_ledger::get_public_key(self.seed_phrase_hd_path.clone()).await;

        match self.format {
            crate::common::OutputFormat::Plaintext => {
                println!(
                    "Seed Phrase HD Path: {}\nImplicit Account ID: {}\nPublic Key: {}",
                    bip32path_to_string(&self.seed_phrase_hd_path),
                    "TBP",
                    "TBP"
                );
            }
            crate::common::OutputFormat::Json => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "seed_phrase_hd_path": bip32path_to_string(&self.seed_phrase_hd_path),
                        "account_id": "TBP",
                        "public_key": "TBP",
                    })).unwrap()
                );
            }
        };
        Ok(())
    }
}
