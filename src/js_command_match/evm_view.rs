#[derive(Debug, Clone, clap::Parser)]
pub struct EvmViewArgs {
    #[clap(num_args = 0..)]
    evm_view_args: Vec<String>,
}
