#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `stake` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct StateArgs {
    account_id: String,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl StateArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        vec![
            "account".to_owned(),
            "view-account-summary".to_owned(),
            self.account_id.to_owned(),
            "network-config".to_owned(),
            network_config,
            "now".to_owned(),
        ]
    }
}
