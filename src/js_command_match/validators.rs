use crate::js_command_match::constants::NETWORK_ID_ALIASES;

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `validators` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct ValidatorsArgs {
    epoch: String,
    #[clap(long, aliases = NETWORK_ID_ALIASES, default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl ValidatorsArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);
        
        let mut command = vec![
            "validator".to_string(),
            "validators".to_string(),
            "network-config".to_string(),
            network_id,
        ];

        if "current" == &self.epoch {
            command.push("now".to_string())
        } else if "next" == &self.epoch {
            command.push("next".to_string())
        };

        command
    }
}

// All validator related commands should go through the dedicated cargo extension