#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `create-account` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct DeleteAccountArgs {
    account_id: String,
    beneficiary_id: String,
    // TODO Rename ledger variables the same way as it is for create_account
    #[clap(long, aliases = ["signWithLedger", "useLedgerKey"], default_value_t = false, conflicts_with = "public_key")]
    use_ledger: bool,
    #[clap(long, aliases = ["ledgerPath"], default_missing_value = Some("44'/397'/0'/0'/1'"), num_args=0..=1)]
    ledger_path: Option<String>,
    #[clap(long, aliases = ["network_id", "networkId"], default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl DeleteAccountArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config.to_owned());

        let mut command = vec![
            "account".to_owned(),
            "delete-account".to_owned(),
            self.account_id.to_owned(),
            "beneficiary".to_owned(),
            self.beneficiary_id.to_owned()
        ];

        command.push("network-config".to_owned());
        command.push(network_id);

        if self.use_ledger {
            command.push("sign-with-ledger".to_owned());
            command.push("--seed-phrase-hd-path".to_owned());
            command.push(self.ledger_path.to_owned().unwrap_or_default());
        } else {
            command.push("sign-with-legacy-keychain".to_owned());
        }

        command.push("send".to_owned());
        
        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;
 
    #[test]
    fn delete_account_using_ledger_testnet() {
        let delete_args = DeleteAccountArgs {
            account_id: "bob.testnet".to_string(),
            beneficiary_id: "alice.testnet".to_string(),
            use_ledger: true,
            ledger_path: None,
            network_id: None,
            _unknown_args: [].to_vec(),
        };
        let result = DeleteAccountArgs::to_cli_args(&delete_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            format!(
                "account delete-account {} beneficiary {} sign-with-ledger --seed-phrase-hd-path  network-config testnet send",
                delete_args.account_id,
                delete_args.beneficiary_id
            )
        )
    }

    #[test]
    fn delete_account_using_ledger_mainnet() {
        let delete_args = DeleteAccountArgs {
            account_id: "bob.testnet".to_string(),
            beneficiary_id: "alice.testnet".to_string(),
            use_ledger: true,
            ledger_path: None,
            network_id: Some("mainnet".to_string()),
            _unknown_args: [].to_vec(),
        };
        let result = DeleteAccountArgs::to_cli_args(&delete_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            format!(
                "account delete-account {} beneficiary {} sign-with-ledger --seed-phrase-hd-path  network-config mainnet send",
                delete_args.account_id,
                delete_args.beneficiary_id
            )
        )
    }
    
    #[test]
    fn delete_account_using_keychain_testnet() {
        let delete_args = DeleteAccountArgs {
            account_id: "bob.testnet".to_string(),
            beneficiary_id: "alice.testnet".to_string(),
            use_ledger: false,
            ledger_path: None,
            network_id: None,
            _unknown_args: [].to_vec(),
        };
        let result = DeleteAccountArgs::to_cli_args(&delete_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            format!(
                "account delete-account {} beneficiary {} sign-with-legacy-keychain network-config testnet send",
                delete_args.account_id,
                delete_args.beneficiary_id
            )
        )
    }
}