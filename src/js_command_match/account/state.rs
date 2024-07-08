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
    use super::*;
    use clap::Parser;

    #[test]
    fn state_testnet() {
        let account_id = "contract.testnet";
        let state_args = StateArgs::parse_from(&["near", account_id]);
        let result = StateArgs::to_cli_args(&state_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            format!("account view-account-summary {account_id} network-config testnet now",)
        )
    }

    #[test]
    fn state_mainnet() {
        let account_id = "contract.testnet";
        let network_id = "mainnet";

        for network_alias in NETWORK_ID_ALIASES {
            let state_args = StateArgs::parse_from(&[
                "near",
                account_id,
                &format!("--{network_alias}"),
                network_id,
            ]);
            let result = StateArgs::to_cli_args(&state_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!(
                    "account view-account-summary {account_id} network-config {network_id} now",
                )
            )
        }
    }
}
