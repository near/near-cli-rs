#[derive(Debug, Clone, clap::Parser)]
pub struct GenerateKeyArgs {
    #[clap(long, aliases = ["account_id", "accountId"])]
    account_id: Option<String>,
    #[clap(long, aliases = ["use_ledger_key", "useLedgerKey"])]
    use_ledger_key: Option<String>,
}

impl GenerateKeyArgs {
    pub fn to_cli_args(&self) -> color_eyre::eyre::Result<Vec<String>> {
        let config = crate::common::get_config_toml()?;
        let folder_path = shellexpand::tilde(
            format!("{}/implicit", config.credentials_home_dir.to_string_lossy()).as_str(),
        )
        .as_ref()
        .parse()?;
        Ok(vec![
            "account".to_owned(),
            "create-account".to_owned(),
            "fund-later".to_owned(),
            "use-auto-generation".to_owned(),
            "save-to-folder".to_owned(),
            folder_path,
        ])
    }
}
