#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `send` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct SendArgs {
    sender_account_id: String,
    receiver_account_id: String,
    amount: String,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl SendArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        vec![
            "tokens".to_owned(),
            self.sender_account_id.to_owned(),
            "send-near".to_owned(),
            self.receiver_account_id.to_owned(),
            format!("{} NEAR", self.amount),
            "network-config".to_owned(),
            network_config,
            "sign-with-keychain".to_owned(),
            "send".to_owned(),
        ]
    }
}
