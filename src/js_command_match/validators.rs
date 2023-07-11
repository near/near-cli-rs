#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `validators` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct ValidatorsArgs {
    epoch: String,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl ValidatorsArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let epoch = if let "current" = self.epoch.as_ref() {
            "now"
        } else {
            &self.epoch
        };
        vec![
            "validator".to_owned(), //XXX extension name without "near-"
            "validators".to_owned(),
            "network-config".to_owned(),
            network_config,
            epoch.to_owned(),
        ]
    }
}
