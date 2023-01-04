#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `proposals` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct ProposalsArgs {
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}
