use crate::js_command_match::constants::NETWORK_ID_ALIASES;

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `legacy` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct LoginArgs {
    #[clap(long, aliases = NETWORK_ID_ALIASES)]
    network_id: Option<String>,
}

impl LoginArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);

        let command = vec![
            "account".to_string(),
            "import-account".to_string(),
            "using-web-wallet".to_string(),
            "network-config".to_string(),
            network_id,
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
    fn login() {
        for (input, expected_output) in [
            (
                "near login".to_string(),
                "account import-account using-web-wallet network-config testnet".to_string()
            ),
            (
                format!("near login --{} testnet", NETWORK_ID_ALIASES[0]),
                "account import-account using-web-wallet network-config testnet".to_string()
            ),
            (
                format!("near login --{} mainnet", NETWORK_ID_ALIASES[1]),
                "account import-account using-web-wallet network-config mainnet".to_string()
            )
        ] {
            let input_cmd = shell_words::split(&input).expect("Input command must be a valid shell command");
            let JsCmd::Login(login_args) = JsCmd::parse_from(&input_cmd) else {
                panic!("Login command was expected, but something else was parsed out from {input}");
            };
            assert_eq!(
                shell_words::join(LoginArgs::to_cli_args(&login_args, "testnet".to_string())),
                expected_output
            );
        }
    }
}
