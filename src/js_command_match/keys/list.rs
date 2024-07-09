use crate::js_command_match::constants::NETWORK_ID_ALIASES;

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `keys` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct KeysArgs {
    account_id: String,
    #[clap(long, aliases = NETWORK_ID_ALIASES)]
    network_id: Option<String>,
}

impl KeysArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);

        let command = vec![
            "account".to_string(),
            "list-keys".to_string(),
            self.account_id.to_owned(),
            "network-config".to_string(),
            network_id,
            "now".to_string(),
        ];

        command
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::JsCmd;
    use super::*;
    use clap::Parser;

    #[test]
    fn list_keys() {
        for (input, expected_output) in [
            (
                "near list-keys bob.testnet".to_string(),
                "account list-keys bob.testnet network-config testnet now".to_string(),
            ),
            (
                format!(
                    "near list-keys bob.testnet --{} testnet",
                    NETWORK_ID_ALIASES[0]
                ),
                "account list-keys bob.testnet network-config testnet now".to_string(),
            ),
            (
                format!(
                    "near list-keys bob.testnet --{} mainnet",
                    NETWORK_ID_ALIASES[1]
                ),
                "account list-keys bob.testnet network-config mainnet now".to_string(),
            ),
        ] {
            let input_cmd =
                shell_words::split(&input).expect("Input command must be a valid shell command");
            let JsCmd::ListKeys(keys_args) = JsCmd::parse_from(&input_cmd) else {
                panic!(
                    "ListKeys command was expected, but something else was parsed out from {input}"
                );
            };
            assert_eq!(
                shell_words::join(KeysArgs::to_cli_args(&keys_args, "testnet".to_string())),
                expected_output
            );
        }
    }
}
