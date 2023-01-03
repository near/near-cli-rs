#[derive(Debug, Clone, clap::Parser)]
pub struct EvmDevInitArgs {
    #[clap(num_args = 0..)]
    evm_dev_init_args: Vec<String>,
}
