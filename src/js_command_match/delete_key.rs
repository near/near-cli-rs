#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `delete-key` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct DeleteKeyArgs {
    account_id: String,
    access_key: String,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl DeleteKeyArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        vec![
            "account".to_owned(),
            "delete-key".to_owned(),
            self.account_id.to_owned(),
            self.access_key.to_owned(),
            "network-config".to_owned(),
            network_config,
            "sign-with-keychain".to_owned(),
            "send".to_owned(),
        ]
    }
}
