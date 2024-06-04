#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `validators` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct ValidatorsArgs {
    epoch: String,
    #[clap(long, aliases = ["network_id", "networkId"], default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl ValidatorsArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config.to_owned());
        
        let mut command = vec![
            "validator".to_owned(),
            "validators".to_owned(),
            "network-config".to_owned(),
            network_id,
        ];

        if "current" == &self.epoch {
            command.push("now".to_owned())
        } else if "next" == &self.epoch {
            command.push("next".to_owned())
        };

        command
    }
}
