use crate::js_command_match::constants::NETWORK_ID_ALIASES;

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `view` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct ViewArgs {
    contract_name: String,
    method_name: String,
    #[clap(default_value = "")]
    args: String,
    #[clap(long, aliases = NETWORK_ID_ALIASES)]
    network_id: Option<String>,
}

impl ViewArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);

        let command = vec![
            "contract".to_string(),
            "call-function".to_string(),
            "as-read-only".to_string(),
            self.contract_name.to_owned(),
            self.method_name.to_owned(),
            "text-args".to_string(),
            self.args.to_owned(),
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
    fn view() {
        let args = "{\"account_id\": \"bob.testnet\"}";

        for (input, expected_output) in [
            (
                format!("near view counter.near-examples.testnet get '{args}'"),
                format!("contract call-function as-read-only counter.near-examples.testnet get text-args '{args}' network-config testnet now")
            ),
            (
                format!("near view counter.near-examples.testnet get '{args}' --{} testnet", NETWORK_ID_ALIASES[0]),
                format!("contract call-function as-read-only counter.near-examples.testnet get text-args '{args}' network-config testnet now")
            ),
            (
                format!("near view counter.near-examples.testnet get '{args}' --{} mainnet", NETWORK_ID_ALIASES[1]),
                format!("contract call-function as-read-only counter.near-examples.testnet get text-args '{args}' network-config mainnet now")
            ),
        ] {
            let input_cmd = shell_words::split(&input).expect("Input command must be a valid shell command");
            let JsCmd::View(view_args) = JsCmd::parse_from(&input_cmd) else {
                panic!("View command was expected, but something else was parsed out from {input}");
            };
            assert_eq!(
                shell_words::join(ViewArgs::to_cli_args(&view_args, "testnet".to_string())),
                expected_output
            );
        }
    }
}
