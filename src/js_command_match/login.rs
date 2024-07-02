use crate::js_command_match::constants::{
    WALLET_URL_ALIASES,
    DEFAULT_WALLET_URL,
    NETWORK_ID_ALIASES,
};

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `legacy` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct LoginArgs {
    #[clap(long, aliases = WALLET_URL_ALIASES, default_value = DEFAULT_WALLET_URL)]
    wallet_url: String,
    #[clap(long, aliases = NETWORK_ID_ALIASES, default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
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
    fn login_testnet() {
        let network_id = "testnet";

        for i in 0..NETWORK_ID_ALIASES.len() {
            let network_id_parameter_alias = &format!("--{}", &NETWORK_ID_ALIASES[i]);
            let login_args = LoginArgs::parse_from(&[
                "near",
                network_id_parameter_alias,
                network_id
            ]);
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

    #[test]
    fn login_mainnet() {
        let network_id = "mainnet";

        let network_id_parameter_alias = &format!("--{}", &NETWORK_ID_ALIASES[0]);
        let login_args = LoginArgs::parse_from(&[
            "near",
            network_id_parameter_alias,
            network_id
        ]);
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