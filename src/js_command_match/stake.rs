#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `stake` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct StakeArgs {
    account_id: String,
    staking_key: String,
    amount: String,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl StakeArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        vec![
            "validator".to_owned(),
            "staking".to_owned(),
            "stake-proposal".to_owned(),
            self.account_id.to_owned(),
            self.staking_key.to_owned(),
            format!("{} NEAR", self.amount),
            "network-config".to_owned(),
            network_config,
            "sign-with-keychain".to_owned(),
            "send".to_owned(),
        ]
    }
}
