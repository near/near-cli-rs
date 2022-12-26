#[derive(Debug, Clone, clap::Parser)]
pub struct DevDeployArgs {
    #[clap(long = "wasmFile")]
    wasm_file: String,
    #[clap(long = "initFunction")]
    init_function: Option<String>,
    #[clap(long = "initArgs")]
    init_args: Option<String>,
    #[clap(long = "initGas", default_value_t = 30_000_000_000_000)]
    init_gas: u64,
    #[clap(long = "initDeposit", default_value = "0")]
    init_deposit: String,
    #[clap(long = "initialBalance", default_value = "100")]
    initial_balance: String,
    #[clap(long = "force", default_value_t = false)]
    force: bool,
}
