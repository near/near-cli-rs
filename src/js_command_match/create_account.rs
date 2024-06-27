#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `create-account` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct CreateAccountArgs {
    account_id: String,
    #[clap(long, aliases = ["masterAccount", "useAccount", "accountId"], default_value = None)]
    master_account: Option<String>,
    #[clap(long, aliases = ["useFaucet"], default_value_t = false, conflicts_with = "master_account")]
    use_faucet: bool,
    #[clap(long, aliases = ["seedPhrase"], default_value = None, conflicts_with = "public_key")]
    seed_phrase: Option<String>,
    #[clap(long, aliases = ["publicKey"], default_value = None)]
    public_key: Option<String>,
    #[clap(long, aliases = ["signWithLedger", "useLedgerKey"], default_value_t = false, conflicts_with = "public_key")]
    use_ledger: bool,
    #[clap(long, aliases = ["ledgerPath"], default_missing_value = Some("44'/397'/0'/0'/1'"), num_args=0..=1)]
    ledger_path: Option<String>,
    #[clap(long, aliases = ["initialBalance"], default_value = Some("0.1"))]
    initial_balance: Option<String>,
    #[clap(long, aliases = ["networkId"], default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}
 
impl CreateAccountArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config.to_owned());
        
 
        let mut command = vec!["account".to_owned(), "create-account".to_owned()];
 
        if self.use_faucet {
            command.push("sponsor-by-faucet-service".to_owned());
            command.push(self.account_id.to_owned());
        } else {
            command.push("fund-myself".to_owned());
            command.push(self.account_id.to_owned());
            command.push(format!("{} NEAR", self.initial_balance.to_owned().unwrap_or_default()));
        }
 
        if self.use_ledger {
            command.push("use-ledger".to_owned());
 
            // if self.ledger_path
            // use as path what comes in self.ledger_path
        };
 
        if self.seed_phrase.is_some() {
            command.push("use-manually-provided-seed-phrase".to_owned());
            command.push(self.seed_phrase.clone().unwrap());
        };

        if self.public_key.is_some() {
            command.push("use-manually-provided-public-key".to_owned());
            command.push(self.public_key.clone().unwrap());
        };

        if !self.seed_phrase.is_some() && !self.public_key.is_some() && !self.use_ledger {
            command.push("autogenerate-new-keypair".to_owned());
            command.push("save-to-keychain".to_owned());
        };
 
        if !self.use_faucet {
            command.push("sign-as".to_owned());
            command.push(
                self.master_account
                    .to_owned()
                    .expect("Valid master account must be provided"),
            );
        };
 
        command.push("network-config".to_owned());
        command.push(network_id);

        if self.use_faucet {
          command.push("create".to_owned());
        } else {
          command.push("sign-with-keychain".to_owned()); // TODO There may be a problem depends how master account was created (by new CLI or old one). New CLI will use just keychain, not legacy one
          command.push("send".to_owned());
        }
 
        command
    }
}
 
#[cfg(test)]
mod tests {
    use super::*;

    use clap::Parser;
 
    #[test]
    fn create_account_using_faucet_testnet_0() {
        let create_account_args = CreateAccountArgs::parse_from(&["near", "bob.testnet", "--useFaucet"]);
        let result = CreateAccountArgs::to_cli_args(&create_account_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            "account create-account sponsor-by-faucet-service bob.testnet autogenerate-new-keypair save-to-keychain network-config testnet create"
        )
    }

    #[test]
    fn create_account_using_faucet_testnet_1() {
        let create_account_args = CreateAccountArgs::parse_from(&["near", "bob.testnet", "--use-faucet"]);
        let result = CreateAccountArgs::to_cli_args(&create_account_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            "account create-account sponsor-by-faucet-service bob.testnet autogenerate-new-keypair save-to-keychain network-config testnet create"
        )
    }
 
    #[test]
    fn create_account_using_master_account_testnet() {
        let create_account_args = CreateAccountArgs::parse_from(&["near", "bob.testnet", "--masterAccount", "alice.testnet", "--initialBalance", "0.1"]);

        let result = CreateAccountArgs::to_cli_args(&create_account_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            "account create-account fund-myself bob.testnet 0.1 NEAR autogenerate-new-keypair save-to-keychain sign-as alice.testnet network-config testnet sign-with-keychain send".to_string()
        )
    }
 
    #[test]
    fn create_account_using_master_account_with_init_balance_testnet() {
        let create_account_args = CreateAccountArgs {
            account_id: "bob.testnet".to_string(),
            master_account: Some("alice.testnet".to_string()),
            use_faucet: false,
            seed_phrase: None,
            public_key: None,
            use_ledger: false,
            ledger_path: None,
            initial_balance: Some("1".to_owned()),
            network_id: None,
            _unknown_args: [].to_vec(),
        };
        let result = CreateAccountArgs::to_cli_args(&create_account_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            "account create-account fund-myself bob.testnet \"1 NEAR\" autogenerate-new-keypair save-to-legacy-keychain sign-as alice.testnet network-config testnet create".to_string()
        )
    }
}
