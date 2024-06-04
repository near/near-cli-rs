#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `send` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct SendArgs {
    pub sender_account_id: String,
    pub receiver_account_id: String,
    pub amount: String,
    #[clap(long, aliases = ["signWithLedger", "useLedgerKey"], default_value_t = false, conflicts_with = "public_key")]
    use_ledger: bool,
    #[clap(long, aliases = ["ledgerPath"], default_missing_value = Some("44'/397'/0'/0'/1'"), num_args=0..=1)]
    ledger_path: Option<String>,
    #[clap(long, aliases = ["network_id", "networkId"], default_value=None)]
    pub network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    pub _unknown_args: Vec<String>,
}

impl SendArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config.to_owned());

        let mut command = vec![
            "tokens".to_owned(),
            self.sender_account_id.to_owned(),
            "send-near".to_owned(),
            self.receiver_account_id.to_owned(),
            format!("{} NEAR", self.amount),
            "network-config".to_owned(),
            network_id,
        ];
        
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
    fn send_with_ledger_key_and_custom_path_testnet() {
        let send_args = SendArgs {
            sender_account_id: "bob.testnet".to_string(),
            receiver_account_id: "alice.testnet".to_string(),
            amount: "1".to_string(),
            use_ledger: true,
            ledger_path: Some("m/44'/397'/0'/0'/2'".to_string()),
            network_id: None,
            _unknown_args: [].to_vec(),
        };
        let result = SendArgs::to_cli_args(&send_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            "tokens bob.testnet send-near alice.testnet 1 NEAR network-config testnet sign-with-ledger --seed-phrase-hd-path m/44'/397'/0'/0'/2' send".to_string()
        )
    }

    #[test]
    fn send_with_ledger_key_and_custom_path_mainnet() {
        let send_args = SendArgs {
            sender_account_id: "bob.testnet".to_string(),
            receiver_account_id: "alice.testnet".to_string(),
            amount: "1".to_string(),
            use_ledger: true,
            ledger_path: Some("m/44'/397'/0'/0'/2'".to_string()),
            network_id: Some("mainnet".to_string()),
            _unknown_args: [].to_vec(),
        };
        let result = SendArgs::to_cli_args(&send_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            "tokens bob.testnet send-near alice.testnet 1 NEAR network-config mainnet sign-with-ledger --seed-phrase-hd-path m/44'/397'/0'/0'/2' send".to_string()
        )
    }

    #[test]
    fn send_with_keychain_testnet() {
        let send_args = SendArgs {
            sender_account_id: "bob.testnet".to_string(),
            receiver_account_id: "alice.testnet".to_string(),
            amount: "1".to_string(),
            use_ledger: false,
            ledger_path: None,
            network_id: None,
            _unknown_args: [].to_vec(),
        };
        let result = SendArgs::to_cli_args(&send_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            "tokens bob.testnet send-near alice.testnet 1 NEAR network-config testnet sign-with-legacy-keychain send".to_string()
        )
    }
}