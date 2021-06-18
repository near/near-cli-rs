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
        let public_key = match near_ledger::get_public_key(self.seed_phrase_hd_path.clone()).await {
            Ok(public_key) => public_key,
            Err(near_ledger_error) => {
                println!(
                    "An error occurred while trying to get PublicKey from Ledger device: {:?}",
                    near_ledger_error
                );
                return Ok(());
            }
        };

        let implicit_account_id = hex::encode(&public_key);

        match self.format {
            crate::common::OutputFormat::Plaintext => {
                println!(
                    "Seed Phrase HD Path: {}\nImplicit Account ID: {}\nPublic Key: {}",
                    bip32path_to_string(&self.seed_phrase_hd_path),
                    implicit_account_id,
                    format!("ed25519:{}", bs58::encode(&public_key).into_string()),
                );
            }
            crate::common::OutputFormat::Json => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "seed_phrase_hd_path": bip32path_to_string(&self.seed_phrase_hd_path),
                        "account_id": implicit_account_id,
                        "public_key": format!("ed25519:{}" ,bs58::encode(&public_key).into_string()),
                    })).unwrap()
                );
            }
        };
        Ok(())
    }
}
