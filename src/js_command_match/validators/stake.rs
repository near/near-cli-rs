#[derive(Debug, Clone, clap::Parser)]
pub struct StakeArgs {
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}
