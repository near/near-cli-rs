#[derive(Debug, Clone, clap::Parser)]
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
}
