#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `validators` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct ValidatorsArgs {
    epoch: String,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl ValidatorsArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let mut output_vec = vec![
            "validator".to_owned(),
            "validators".to_owned(),
            "network-config".to_owned(),
            network_config,
        ];
        if "current" == &self.epoch {
            output_vec.push("now".to_owned())
        } else if "next" == &self.epoch {
            output_vec.push("next".to_owned())
        };
        output_vec
    }
}
