use crate::js_command_match::constants::NETWORK_ID_ALIASES;

#[derive(Debug, Clone, clap::Parser)]
pub struct StateArgs {
    account_id: String,
    #[clap(long, aliases = NETWORK_ID_ALIASES)]
    network_id: Option<String>,
}

impl StateArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);

        let command = vec![
            "account".to_string(),
            "view-account-summary".to_string(),
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
    fn state() {
        for (input, expected_output) in [
            (
                "near state contract.testnet".to_string(),
                "account view-account-summary contract.testnet network-config testnet now".to_string()
            ),
            (
                format!("near state contract.testnet --{} testnet", NETWORK_ID_ALIASES[0]),
                "account view-account-summary contract.testnet network-config testnet now".to_string()
            ),
            (
                format!("near state contract.testnet --{} mainnet", NETWORK_ID_ALIASES[1]),
                "account view-account-summary contract.testnet network-config mainnet now".to_string()
            ),
        ] {
            let input_cmd = shell_words::split(&input).expect("Input command must be a valid shell command");
            let JsCmd::State(state_args) = JsCmd::parse_from(&input_cmd) else {
                panic!("State command was expected, but something else was parsed out from {input}");
            };
            assert_eq!(
                shell_words::join(StateArgs::to_cli_args(&state_args, "testnet".to_string())),
                expected_output
            );
        }
    }
}
