use color_eyre::owo_colors::OwoColorize;

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `validators` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct ValidatorsArgs {
    #[arg(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

#[derive(Debug, Clone, clap::Parser)]
pub struct StakeArgs {
    #[arg(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

const DEPRECATED: &str = "The command you tried to run has been moved into its own CLI extension called near-validator.\nPlease, follow the installation instructions here: https://github.com/near-cli-rs/near-validator-cli-rs/blob/master/README.md";

impl ValidatorsArgs {
    pub fn to_cli_args(&self, _network_config: String) -> Vec<String> {
        eprintln!("\n{}\n", DEPRECATED.to_string().yellow());
        vec!["near-validator".to_string()]
    }
}

impl StakeArgs {
    pub fn to_cli_args(&self, _network_config: String) -> Vec<String> {
        eprintln!("\n{}\n", DEPRECATED.to_string().yellow());
        vec!["near-validator".to_string()]
    }
}
