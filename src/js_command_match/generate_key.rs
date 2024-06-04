#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `generate-key` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct GenerateKeyArgs {
    account_id: Option<String>,
    #[clap(long, aliases = ["seed_phrase", "seedPhrase"], default_value = None, conflicts_with = "use_ledger")]
    seed_phrase: Option<String>,
    #[clap(long, aliases = ["signWithLedger", "useLedgerKey"], default_value_t = false, conflicts_with = "public_key")]
    use_ledger: bool,
    #[clap(long, aliases = ["ledgerPath"], default_missing_value = Some("44'/397'/0'/0'/1'"), num_args=0..=1)]
    ledger_path: Option<String>,
    #[clap(long, aliases = ["network_id", "networkId"], default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl GenerateKeyArgs {
    pub fn to_cli_args(&self, network_config: String) -> color_eyre::eyre::Result<Vec<String>> {
        let network_id = self.network_id.clone().unwrap_or(network_config.to_owned());
        
        let mut command = vec![
            "account".to_owned(),
            "create-account".to_owned(),
            "fund-later".to_owned(),
        ];
          
        let config = crate::config::Config::get_config_toml()?;
        let mut generation_method = "use-auto-generation".to_string();

        if self.use_ledger {
            generation_method = "use-ledger".to_string();
        }

        let folder_path;

        if let Some(account_id) = self.account_id.as_deref() {
            folder_path = shellexpand::tilde(
                format!(
                    "{}/{}/{}",
                    config.credentials_home_dir.to_string_lossy(),
                    network_id,
                    account_id
            )
            .as_str()).as_ref().parse()?;
        } else {
            folder_path = shellexpand::tilde(format!("{}/implicit", config.credentials_home_dir.to_string_lossy()).as_str()).as_ref().parse()?;
        }

        if let Some(seed_phrase) = self.seed_phrase.as_deref() {
            command.push("use-seed-phrase".to_owned());
            command.push(seed_phrase.to_owned());
            command.push("--seed-phrase-hd-path".to_owned());
            command.push("m/44'/397'/0'".to_owned());
            command.push("save-to-folder".to_owned());
            command.push(folder_path);

            return Ok(command);
        }
        
        command.push(generation_method);
        command.push("save-to-folder".to_owned());
        command.push(folder_path);

        Ok(command)
    }
}
