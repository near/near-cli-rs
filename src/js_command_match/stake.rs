use crate::js_command_match::constants::NETWORK_ID_ALIASES;

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `stake` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct StakeArgs {
    account_id: String,
    staking_key: String,
    amount: String,
    #[clap(long, aliases = NETWORK_ID_ALIASES, default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl StakeArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);

        let command = vec![
            "validator".to_string(),
            "staking".to_string(),
            "stake-proposal".to_string(),
            self.account_id.to_owned(),
            self.staking_key.to_owned(),
            format!("{} NEAR", self.amount),
            "network-config".to_string(),
            network_id,
            "sign-with-keychain".to_string(),
            "send".to_string(),
        ];

        command
    }
}

// All validator related commands should go through the dedicated cargo extension