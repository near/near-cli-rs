use crate::js_command_match::constants::{
    DEFAULT_SEED_PHRASE_PATH, LEDGER_PATH_ALIASES, NETWORK_ID_ALIASES, SIGN_WITH_LEDGER_ALIASES,
};

#[derive(Debug, Clone, clap::Parser)]
#[clap(alias("send-near"))]
pub struct SendArgs {
    pub sender: String,
    pub receiver: String,
    pub amount: String,
    #[clap(long, aliases = SIGN_WITH_LEDGER_ALIASES, default_value_t = false)]
    sign_with_ledger: bool,
    #[clap(long, aliases = LEDGER_PATH_ALIASES, default_value = Some(DEFAULT_SEED_PHRASE_PATH))]
    ledger_path: Option<String>,
    #[clap(long, aliases = NETWORK_ID_ALIASES, default_value=None)]
    pub network_id: Option<String>,
}

impl SendArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);

        let mut command = vec![
            "tokens".to_string(),
            self.sender.to_owned(),
            "send-near".to_string(),
            self.receiver.to_owned(),
            format!("{} NEAR", self.amount),
            "network-config".to_string(),
            network_id,
        ];

        if self.sign_with_ledger {
            command.push("sign-with-ledger".to_string());
            command.push("--seed-phrase-hd-path".to_string());
            command.push(self.ledger_path.to_owned().unwrap_or_default());
        } else {
            command.push("sign-with-keychain".to_string());
        }

        command.push("send".to_string());
        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn send_with_ledger_key_and_custom_path_testnet() {
        let sender_account_id = "bob.testnet";
        let receiver_account_id = "alice.testnet";
        let amount = "1";
        let custom_ledger_path = "m/44'/397'/0'/0'/2'";

        for i in 0..SIGN_WITH_LEDGER_ALIASES.len() {
            let use_ledger_parameter_alias = &format!("--{}", &SIGN_WITH_LEDGER_ALIASES[i]);

            for j in 0..LEDGER_PATH_ALIASES.len() {
                let ledger_path_parameter_alias = &format!("--{}", &LEDGER_PATH_ALIASES[j]);
                let send_args = SendArgs::parse_from(&[
                    "near",
                    sender_account_id,
                    receiver_account_id,
                    amount,
                    use_ledger_parameter_alias,
                    ledger_path_parameter_alias,
                    custom_ledger_path,
                ]);
                let result = SendArgs::to_cli_args(&send_args, "testnet".to_string());
                assert_eq!(
                    result.join(" "),
                    format!(
                        "tokens {} send-near {} {} NEAR network-config testnet sign-with-ledger --seed-phrase-hd-path {} send",
                        sender_account_id,
                        receiver_account_id,
                        amount,
                        custom_ledger_path
                    )
                )
            }
        }
    }

    #[test]
    fn send_with_ledger_key_and_custom_path_mainnet() {
        let sender_account_id = "bob.testnet";
        let receiver_account_id = "alice.testnet";
        let amount = "1";
        let custom_ledger_path = "m/44'/397'/0'/0'/2'";
        let network_id = "mainnet";
        let use_ledger_parameter_alias = &format!("--{}", &SIGN_WITH_LEDGER_ALIASES[0]);
        let ledger_path_parameter_alias = &format!("--{}", &LEDGER_PATH_ALIASES[0]);

        for i in 0..NETWORK_ID_ALIASES.len() {
            let network_id_parameter_alias = &format!("--{}", &NETWORK_ID_ALIASES[i]);
            let send_args = SendArgs::parse_from(&[
                "near",
                sender_account_id,
                receiver_account_id,
                amount,
                use_ledger_parameter_alias,
                ledger_path_parameter_alias,
                custom_ledger_path,
                network_id_parameter_alias,
                network_id,
            ]);
            let result = SendArgs::to_cli_args(&send_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!(
                    "tokens {} send-near {} {} NEAR network-config {} sign-with-ledger --seed-phrase-hd-path {} send",
                    sender_account_id,
                    receiver_account_id,
                    amount,
                    network_id,
                    custom_ledger_path
                )
            )
        }

        let send_args = SendArgs {
            sender: "bob.testnet".to_string(),
            receiver: "alice.testnet".to_string(),
            amount: "1".to_string(),
            sign_with_ledger: true,
            ledger_path: Some("m/44'/397'/0'/0'/2'".to_string()),
            network_id: Some("mainnet".to_string()),
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
            sender: "bob.testnet".to_string(),
            receiver: "alice.testnet".to_string(),
            amount: "1".to_string(),
            sign_with_ledger: false,
            ledger_path: None,
            network_id: None,
        };
        let result = SendArgs::to_cli_args(&send_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            "tokens bob.testnet send-near alice.testnet 1 NEAR network-config testnet sign-with-keychain send".to_string()
        )
    }
}
