#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `create-account` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct DeleteAccountArgs {
    account_id: String,
    beneficiary_id: String,
    #[clap(long, aliases = ["use_ledger_key", "useLedgerKey", "signWithLedger"], default_missing_value = Some("44'/397'/0'/0'/1'"), num_args=0..=1)]
    use_ledger_key: Option<String>,
    #[clap(long, aliases = ["ledger_path", "ledgerPath"], default_value = Some("44'/397'/0'/0'/1'"))]
    ledger_path: Option<String>,
    #[clap(long, aliases = ["network_id", "networkId"], default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl DeleteAccountArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config.to_owned());

        if self.use_ledger_key.is_some() {
            return vec![
                "account".to_owned(),
                "delete-account".to_owned(),
                self.account_id.to_owned(),
                "beneficiary".to_owned(),
                self.beneficiary_id.to_owned(),
                "network-config".to_owned(),
                network_id,
                "sign-with-ledger".to_owned(),
                "--seed-phrase-hd-path".to_owned(),
                self.ledger_path.to_owned().unwrap_or_default(),
                "send".to_owned(),
            ];
        }
        return vec![
            "account".to_owned(),
            "delete-account".to_owned(),
            self.account_id.to_owned(),
            "beneficiary".to_owned(),
            self.beneficiary_id.to_owned(),
            "network-config".to_owned(),
            network_id,
            "sign-with-legacy-keychain".to_owned(),
            "send".to_owned(),
        ];
    }
}
