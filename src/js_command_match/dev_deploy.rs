#[derive(Debug, Clone, clap::Parser)]
pub struct DevDeployArgs {
    wasm_file: String,
    init_function: String,
    init_args: String,
    init_gas: String,
    init_deposit: String,
    initial_balance: String,
    force: String,
}
