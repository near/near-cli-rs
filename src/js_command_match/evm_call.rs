#[derive(Debug, Clone, clap::Parser)]
pub struct EvmCallArgs {
    #[clap(num_args = 0..)]
    evm_call_args: Vec<String>,
}
