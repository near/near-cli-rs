#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `view-state` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct ViewStateArgs {
    account_id: String,
    #[clap(long, default_value = "")]
    prefix: String,
    #[clap(long, aliases = ["block_id", "blockId"], default_value = "0")]
    block_id: String,
    #[clap(long, default_value = "final")]
    finality: String,
    #[clap(long, default_value = "false")]
    utf8: String,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}
