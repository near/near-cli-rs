#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `proposals` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct ProposalsArgs {
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl ProposalsArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        vec![
            "validator".to_owned(),
            "proposals".to_owned(),
            "network-config".to_owned(),
            network_config,
        ]
    }
}
