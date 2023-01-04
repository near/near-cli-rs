#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `evm-call` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct EvmCallArgs {
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    evm_call_args: Vec<String>,
}
