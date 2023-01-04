#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `evm-view` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct EvmViewArgs {
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    evm_view_args: Vec<String>,
}
