#[derive(Debug, Clone, clap::Parser)]
pub struct CreateAccountArgs {
    account_id: String,
    #[clap(long, aliases = ["master_account", "masterAccount"])]
    master_account: String,
    #[clap(long, aliases = ["public_key", "publicKey"], default_value = None, conflicts_with = "new_ledger_key")]
    public_key: Option<String>,
    #[clap(long, aliases = ["new_ledger_key", "newLedgerKey"], default_missing_value = Some("44'/397'/0'/0'/1'"), num_args=0..=1)]
    new_ledger_key: Option<String>,
    #[clap(long, aliases = ["initial_balance", "initialBalance"], default_value = "100")]
    initial_balance: String,
}

impl CreateAccountArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        if self.new_ledger_key.is_some() {
            return vec![
                "account".to_owned(),
                "create-account".to_owned(),
                "fund-myself".to_owned(),
                self.account_id.to_owned(),
                format!("{} NEAR", self.initial_balance),
                "use-ledger".to_owned(),
                "sign-as".to_owned(),
                self.master_account.to_owned(),
                "network-config".to_owned(),
                network_config,
                "sign-with-keychain".to_owned(),
                "send".to_owned(),
            ];
        }
        if let Some(public_key) = self.public_key.as_deref() {
            vec![
                "account".to_owned(),
                "create-account".to_owned(),
                "fund-myself".to_owned(),
                self.account_id.to_owned(),
                format!("{} NEAR", self.initial_balance),
                "use-manually-provided-public-key".to_owned(),
                public_key.to_owned(),
                "sign-as".to_owned(),
                self.master_account.to_owned(),
                "network-config".to_owned(),
                network_config,
                "sign-with-keychain".to_owned(),
                "send".to_owned(),
            ]
        } else {
            vec![
                "account".to_owned(),
                "create-account".to_owned(),
                "fund-myself".to_owned(),
                self.account_id.to_owned(),
                format!("{} NEAR", self.initial_balance),
                "autogenerate-new-keypair".to_owned(),
                "save-to-keychain".to_owned(),
                "sign-as".to_owned(),
                self.master_account.to_owned(),
                "network-config".to_owned(),
                network_config,
                "sign-with-keychain".to_owned(),
                "send".to_owned(),
            ]
        }
    }
}
