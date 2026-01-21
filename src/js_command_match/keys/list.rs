use crate::js_command_match::constants::NETWORK_ID_ALIASES;

/// KeysArgs: A legacy command handler for `near keys`.
/// It maps old CLI patterns to the modern structured command system.
#[derive(Debug, Clone, clap::Parser)]
pub struct KeysArgs {
    /// The account ID whose keys are being listed.
    account_id: String,
    
    /// Optional network ID, supports multiple legacy aliases (e.g., --node_url, --networkId).
    #[clap(long, aliases = NETWORK_ID_ALIASES)]
    network_id: Option<String>,
}

impl KeysArgs {
    /// Maps legacy arguments to the new CLI command vector.
    /// Efficiency: Uses pre-allocated vector capacity to minimize re-allocations.
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.as_deref().unwrap_or(&network_config);

        // Pre-allocating capacity for performance (prevents dynamic resizing)
        let mut command = Vec::with_capacity(7);
        command.extend([
            "account".to_string(),
            "list-keys".to_string(),
            self.account_id.clone(),
            "network-config".to_string(),
            network_id.to_string(),
            "now".to_string(),
        ]);

        command
    }
}



#[cfg(test)]
mod tests {
    use super::super::super::JsCmd;
    use super::*;
    use clap::Parser;

    /// list_keys_test: Validates that legacy inputs produce correct modern commands.
    #[test]
    fn list_keys_test() {
        let test_cases = [
            (
                "near keys bob.testnet",
                "account list-keys bob.testnet network-config testnet now"
            ),
            (
                "near list-keys bob.testnet",
                "account list-keys bob.testnet network-config testnet now"
            ),
            (
                &format!("near list-keys bob.testnet --{} testnet", NETWORK_ID_ALIASES[0]),
                "account list-keys bob.testnet network-config testnet now"
            ),
        ];

        for (input, expected_output) in test_cases {
            let input_tokens = shell_words::split(input)
                .expect("Critical: Input must be a valid shell-escaped command");

            let JsCmd::ListKeys(args) = JsCmd::parse_from(&input_tokens) else {
                panic!("Failed to parse ListKeys from: {}", input);
            };

            let generated_args = args.to_cli_args("testnet".to_string());
            assert_eq!(shell_words::join(generated_args), expected_output);
        }
    }
}
