#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `delete` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct DeleteArgs {
    account_id: String,
    beneficiary_id: String,
    #[clap(long, aliases = ["network_id", "networkId"], default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl DeleteArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config.to_owned());

        let command = vec![
            "account".to_owned(),
            "delete-account".to_owned(),
            self.account_id.to_owned(),
            "beneficiary".to_owned(),
            self.beneficiary_id.to_owned(),
            "network-config".to_owned(),
            network_id,
            "sign-with-keychain".to_owned(),
            "send".to_owned(),
        ];

        command
    }
}
