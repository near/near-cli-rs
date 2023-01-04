#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `stake` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct StakeArgs {
    account_id: String,
    staking_key: String,
    amount: String,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}
