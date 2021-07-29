use std::str::FromStr;

/// Generate a key pair of secret and public keys (use it anywhere you need
/// Ed25519 keys)
#[derive(Debug, Clone, clap::Clap, Clone)]
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
        println!(
            "Please allow getting the PublicKey on Ledger device (HD Path: {})",
            self.seed_phrase_hd_path.to_string(),
        );
        let public_key = near_ledger::get_public_key(self.seed_phrase_hd_path.clone())
            .await
            .map_err(|near_ledger_error| {
                color_eyre::Report::msg(format!(
                    "An error occurred while trying to get PublicKey from Ledger device: {:?}",
                    near_ledger_error
                ))
            })?;

        let implicit_account_id = hex::encode(&public_key);

        match self.format {
            crate::common::OutputFormat::Plaintext => {
                println!(
                    "Seed Phrase HD Path: {}\nImplicit Account ID: {}\nPublic Key: {}",
                    self.seed_phrase_hd_path.to_string(),
                    implicit_account_id,
                    near_crypto::PublicKey::ED25519(near_crypto::ED25519PublicKey::from(
                        public_key.to_bytes(),
                    )),
                );
            }
            crate::common::OutputFormat::Json => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "seed_phrase_hd_path": self.seed_phrase_hd_path.to_string(),
                        "account_id": implicit_account_id,
                        "public_key": near_crypto::PublicKey::ED25519(
                            near_crypto::ED25519PublicKey::from(
                                public_key.to_bytes(),
                            )
                        ),
                    }))
                    .unwrap()
                );
            }
        };
        Ok(())
    }
}
