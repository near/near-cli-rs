#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `validators` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct ValidatorsArgs {
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

#[derive(Debug, Clone, clap::Parser)]
pub struct StakeArgs {
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}
