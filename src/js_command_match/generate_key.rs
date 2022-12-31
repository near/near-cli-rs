#[derive(Debug, Clone, clap::Parser)]
pub struct GenerateKeyArgs {
    account_id: Option<String>,
    #[clap(long, aliases = ["seed_phrase", "seedPhrase"], default_value = None, conflicts_with = "use_ledger_key")]
    seed_phrase: Option<String>,
    #[clap(long, aliases = ["use_ledger_key", "useLedgerKey"], default_missing_value = Some("44'/397'/0'/0'/1'"), num_args=0..=1)]
    use_ledger_key: Option<String>,
}

impl GenerateKeyArgs {
    pub fn to_cli_args(&self, network_config: String) -> color_eyre::eyre::Result<Vec<String>> {
        let config = crate::common::get_config_toml()?;
        let mut generation_method = "use-auto-generation".to_string();
        if self.use_ledger_key.is_some() {
            generation_method = "use-ledger".to_string();
        }
        if let Some(account_id) = self.account_id.as_deref() {
            let folder_path = shellexpand::tilde(
                format!(
                    "{}/{}/{}",
                    config.credentials_home_dir.to_string_lossy(),
                    network_config,
                    account_id
                )
                .as_str(),
            )
            .as_ref()
            .parse()?;
            if let Some(seed_phrase) = self.seed_phrase.as_deref() {
                return Ok(vec![
                    "account".to_owned(),
                    "create-account".to_owned(),
                    "fund-later".to_owned(),
                    "use-seed-phrase".to_owned(),
                    seed_phrase.to_owned(),
                    "--seed-phrase-hd-path".to_owned(),
                    "m/44'/397'/0'".to_owned(),
                    "save-to-folder".to_owned(),
                    folder_path,
                ]);
            }
            return Ok(vec![
                "account".to_owned(),
                "create-account".to_owned(),
                "fund-later".to_owned(),
                generation_method,
                "save-to-folder".to_owned(),
                folder_path,
            ]);
        }
        let folder_path = shellexpand::tilde(
            format!("{}/implicit", config.credentials_home_dir.to_string_lossy()).as_str(),
        )
        .as_ref()
        .parse()?;
        if let Some(seed_phrase) = self.seed_phrase.as_deref() {
            return Ok(vec![
                "account".to_owned(),
                "create-account".to_owned(),
                "fund-later".to_owned(),
                "use-seed-phrase".to_owned(),
                seed_phrase.to_owned(),
                "--seed-phrase-hd-path".to_owned(),
                "m/44'/397'/0'".to_owned(),
                "save-to-folder".to_owned(),
                folder_path,
            ]);
        }
        Ok(vec![
            "account".to_owned(),
            "create-account".to_owned(),
            "fund-later".to_owned(),
            generation_method,
            "save-to-folder".to_owned(),
            folder_path,
        ])
    }
}
