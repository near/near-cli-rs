#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `create-account` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct CreateAccountArgs {
    account_id: String,
    #[clap(long, aliases = ["master_account", "masterAccount", "useAccount", "accountId"], default_value = None)]
    master_account: Option<String>,
    #[clap(long, aliases = ["use_faucet", "useFaucet"], default_value = None, num_args=0, conflicts_with = "master_account")]
    use_faucet: Option<String>,
    #[clap(long, aliases = ["public_key", "publicKey"], default_value = None, conflicts_with = "new_ledger_key")]
    public_key: Option<String>,
    #[clap(long, aliases = ["new_ledger_key", "newLedgerKey"], default_missing_value = Some("44'/397'/0'/0'/1'"), num_args=0..=1)]
    new_ledger_key: Option<String>,
    #[clap(long, aliases = ["initial_balance", "initialBalance"], default_value = "100")]
    initial_balance: String,
    #[clap(long, aliases = ["network_id", "networkId"], default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl CreateAccountArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config.to_owned());

        if self.master_account.is_some() {
            if self.new_ledger_key.is_some() {
                return vec![
                    "account".to_owned(),
                    "create-account".to_owned(),
                    "fund-myself".to_owned(),
                    self.account_id.to_owned(),
                    format!("{} NEAR", self.initial_balance),
                    "use-ledger".to_owned(),
                    "sign-as".to_owned(),
                    self.master_account.to_owned().expect("Valid master account must be provided"),
                    "network-config".to_owned(),
                    network_id,
                    "sign-with-keychain".to_owned(),
                    "send".to_owned(),
                ];
            }
            if let Some(public_key) = self.public_key.as_deref() {
                return vec![
                    "account".to_owned(),
                    "create-account".to_owned(),
                    "fund-myself".to_owned(),
                    self.account_id.to_owned(),
                    format!("{} NEAR", self.initial_balance),
                    "use-manually-provided-public-key".to_owned(),
                    public_key.to_owned(),
                    "sign-as".to_owned(),
                    self.master_account.to_owned().expect("Valid master account must be provided"),
                    "network-config".to_owned(),
                    network_id,
                    "sign-with-keychain".to_owned(),
                    "send".to_owned(),
                ];
            }
            return vec![
                "account".to_owned(),
                "create-account".to_owned(),
                "fund-myself".to_owned(),
                self.account_id.to_owned(),
                format!("{} NEAR", self.initial_balance),
                "autogenerate-new-keypair".to_owned(),
                "save-to-keychain".to_owned(),
                "sign-as".to_owned(),
                self.master_account.to_owned().expect("Valid master account must be provided"),
                "network-config".to_owned(),
                network_id,
                "sign-with-keychain".to_owned(),
                "send".to_owned(),
            ];
        }

        return vec![
            "account".to_owned(),
            "create-account".to_owned(),
            "sponsor-by-faucet-service".to_owned(),
            self.account_id.to_owned(),
            "autogenerate-new-keypair".to_owned(),
            "save-to-legacy-keychain".to_owned(),
            "network-config".to_owned(),
            network_id,
            "create".to_owned(),
        ];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_account_using_faucet_testnet() {
        let create_account_args = CreateAccountArgs {
            account_id: "bob.testnet".to_string(),
            master_account: None,
            use_faucet: None,
            public_key: None,
            new_ledger_key: None,
            initial_balance: "100".to_string(),
            network_id: None,
            _unknown_args: [].to_vec(),
        };
        let result = CreateAccountArgs::to_cli_args(&create_account_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            "account create-account sponsor-by-faucet-service bob.testnet autogenerate-new-keypair save-to-legacy-keychain network-config testnet create".to_string()
        )
    }

    #[test]
    fn create_account_using_master_account_testnet() {
        let create_account_args = CreateAccountArgs {
            account_id: "bob.testnet".to_string(),
            master_account: Some("alice.testnet".to_string()),
            use_faucet: None,
            public_key: None,
            new_ledger_key: None,
            initial_balance: "100".to_string(),
            network_id: None,
            _unknown_args: [].to_vec(),
        };
        let result = CreateAccountArgs::to_cli_args(&create_account_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            "account create-account fund-myself bob.testnet 100 NEAR autogenerate-new-keypair save-to-keychain sign-as alice.testnet network-config testnet sign-with-keychain send".to_string()
        )
    }

    #[test]
    fn create_account_using_master_account_with_init_balance_testnet() {
        let create_account_args = CreateAccountArgs {
            account_id: "bob.testnet".to_string(),
            master_account: Some("alice.testnet".to_string()),
            use_faucet: None,
            public_key: None,
            new_ledger_key: None,
            initial_balance: "1".to_string(),
            network_id: None,
            _unknown_args: [].to_vec(),
        };
        let result = CreateAccountArgs::to_cli_args(&create_account_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            "account create-account fund-myself bob.testnet 1 NEAR autogenerate-new-keypair save-to-keychain sign-as alice.testnet network-config testnet sign-with-keychain send".to_string()
        )
    }
}