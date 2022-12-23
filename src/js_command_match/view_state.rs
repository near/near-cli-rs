#[derive(Debug, Clone, clap::Parser)]
pub struct ViewStateArgs {
    account_id: String,
    prefix: String,
}
