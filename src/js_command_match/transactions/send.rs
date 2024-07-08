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
    #[clap(long, aliases = LEDGER_PATH_ALIASES, default_value = DEFAULT_SEED_PHRASE_PATH)]
    ledger_path: String,
    #[clap(long, aliases = NETWORK_ID_ALIASES)]
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
            command.push(self.ledger_path.to_owned());
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

        for use_ledger_parameter_alias in SIGN_WITH_LEDGER_ALIASES {
            for ledger_path_parameter_alias in LEDGER_PATH_ALIASES {
                let send_args = SendArgs::parse_from(&[
                    "near",
                    sender_account_id,
                    receiver_account_id,
                    amount,
                    &format!("--{use_ledger_parameter_alias}"),
                    &format!("--{ledger_path_parameter_alias}"),
                    custom_ledger_path,
                ]);
                let result = SendArgs::to_cli_args(&send_args, "testnet".to_string());
                assert_eq!(
                    result.join(" "),
                    format!(
                        "tokens {sender_account_id} send-near {receiver_account_id} {amount} NEAR network-config testnet sign-with-ledger --seed-phrase-hd-path {custom_ledger_path} send",
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

        for network_id_parameter_alias in NETWORK_ID_ALIASES {
            let send_args = SendArgs::parse_from(&[
                "near",
                sender_account_id,
                receiver_account_id,
                amount,
                use_ledger_parameter_alias,
                ledger_path_parameter_alias,
                custom_ledger_path,
                &format!("--{network_id_parameter_alias}"),
                network_id,
            ]);
            let result = SendArgs::to_cli_args(&send_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!(
                    "tokens {sender_account_id} send-near {receiver_account_id} {amount} NEAR network-config {network_id} sign-with-ledger --seed-phrase-hd-path {custom_ledger_path} send",
                )
            )
        }

        let send_args = SendArgs {
            sender: "bob.testnet".to_string(),
            receiver: "alice.testnet".to_string(),
            amount: "1".to_string(),
            sign_with_ledger: true,
            ledger_path: "m/44'/397'/0'/0'/2'".to_string(),
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
            ledger_path: DEFAULT_SEED_PHRASE_PATH.to_string(),
            network_id: None,
        };
        let result = SendArgs::to_cli_args(&send_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            "tokens bob.testnet send-near alice.testnet 1 NEAR network-config testnet sign-with-keychain send".to_string()
        )
    }
}
