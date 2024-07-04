use crate::js_command_match::constants::NETWORK_ID_ALIASES;

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `stake` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct StateArgs {
    account_id: String,
    #[clap(long, aliases = NETWORK_ID_ALIASES, default_value=None)]
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
    use super::*;
    use clap::Parser;

    #[test]
    fn state_testnet() {
        let account_id = "contract.testnet";
        let state_args = StateArgs::parse_from(&["near", account_id]);
        let result = StateArgs::to_cli_args(&state_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            format!(
                "account view-account-summary {} network-config testnet now",
                account_id,
            )
        )
    }

    #[test]
    fn state_mainnet() {
        let account_id = "contract.testnet";
        let network_id = "mainnet";

        for i in 0..NETWORK_ID_ALIASES.len() {
            let network_id_parameter_alias = &format!("--{}", &NETWORK_ID_ALIASES[i]);
            let state_args = StateArgs::parse_from(&[
                "near",
                account_id,
                network_id_parameter_alias,
                network_id,
            ]);
            let result = StateArgs::to_cli_args(&state_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!(
                    "account view-account-summary {} network-config {} now",
                    account_id, network_id,
                )
            )
        }
    }
}
