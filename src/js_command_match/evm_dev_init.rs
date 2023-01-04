#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `evm-dev-init` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct EvmDevInitArgs {
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    evm_dev_init_args: Vec<String>,
}
