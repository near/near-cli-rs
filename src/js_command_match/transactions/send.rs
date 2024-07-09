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
    use super::super::super::JsCmd;
    use super::*;
    use clap::Parser;

    #[test]
    fn send() {
        for (input, expected_output) in [
            (
                format!("near send-near bob.testnet alice.testnet 1 --{}", SIGN_WITH_LEDGER_ALIASES[0]),
                "tokens bob.testnet send-near alice.testnet '1 NEAR' network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/1'\\''' send"
            ),
            (
                format!("near send-near bob.testnet alice.testnet 1 --{}", SIGN_WITH_LEDGER_ALIASES[1]),
                "tokens bob.testnet send-near alice.testnet '1 NEAR' network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/1'\\''' send"
            ),
            (
                format!("near send-near bob.testnet alice.testnet 1 --signWithLedger --{} \"44'/397'/0'/0'/2'\"", LEDGER_PATH_ALIASES[0]),
                "tokens bob.testnet send-near alice.testnet '1 NEAR' network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/2'\\''' send"
            ),
            (
                format!("near send-near bob.testnet alice.testnet 1 --signWithLedger --{} \"44'/397'/0'/0'/2'\"", LEDGER_PATH_ALIASES[1]),
                "tokens bob.testnet send-near alice.testnet '1 NEAR' network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/2'\\''' send"
            ),
            (
                format!("near send-near bob.testnet alice.testnet 1 --{} testnet", NETWORK_ID_ALIASES[0]),
                "tokens bob.testnet send-near alice.testnet '1 NEAR' network-config testnet sign-with-keychain send"
            ),
            (
                format!("near send-near bob.testnet alice.testnet 1 --{} mainnet", NETWORK_ID_ALIASES[1]),
                "tokens bob.testnet send-near alice.testnet '1 NEAR' network-config mainnet sign-with-keychain send"
            ),
        ] {
            let input_cmd = shell_words::split(&input).expect("Input command must be a valid shell command");
            let JsCmd::Send(send_args) = JsCmd::parse_from(&input_cmd) else {
                panic!("Send command was expected, but something else was parsed out from {input}");
            };
            assert_eq!(
                shell_words::join(SendArgs::to_cli_args(&send_args, "testnet".to_string())),
                expected_output
            );
        }
    }
}
