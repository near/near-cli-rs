use crate::js_command_match::constants::NETWORK_ID_ALIASES;

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `legacy` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct LoginArgs {
    #[clap(long, aliases = NETWORK_ID_ALIASES, default_value=None)]
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
    use super::*;
    use clap::Parser;

    #[test]
    fn login() {
        for network_id in vec!["testnet", "mainnet"] {
            let login_args = LoginArgs::parse_from(&["near", "--networkId", network_id]);
            let result = LoginArgs::to_cli_args(&login_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!(
                    "account import-account using-web-wallet network-config {}",
                    network_id,
                )
            );
        }
    }
}
