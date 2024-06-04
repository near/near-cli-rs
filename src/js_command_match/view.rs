#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `view` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct ViewArgs {
    contract_account_id: String,
    method_name: String,
    #[clap(default_value = "")]
    args: String,
    #[clap(long, aliases = ["network_id", "networkId"], default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl ViewArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config.to_owned());

        let command = vec![
            "contract".to_owned(),
            "call-function".to_owned(),
            "as-read-only".to_owned(),
            self.contract_account_id.to_owned(),
            self.method_name.to_owned(),
            "text-args".to_owned(),
            self.args.to_owned(),
            "network-config".to_owned(),
            network_id,
            "now".to_owned(),
        ];

        command
    }
}
