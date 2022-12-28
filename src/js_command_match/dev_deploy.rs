#[derive(Debug, Clone, clap::Parser)]
pub struct DevDeployArgs {
    wasm_file: Option<String>,
    #[clap(long, aliases = ["init_function", "initFunction"])]
    init_function: Option<String>,
    #[clap(long, aliases = ["init_args", "initArgs"], default_value = "{}")]
    init_args: String,
    #[clap(long, aliases = ["init_gas", "initGas"], default_value_t = 30_000_000_000_000)]
    init_gas: u64,
    #[clap(long, aliases = ["init_deposit", "initDeposit"], default_value = "0")]
    init_deposit: String,
    #[clap(long, aliases = ["initial_balance", "initialBalance"], default_value = "100")]
    initial_balance: String,
    #[clap(long, default_value = "false")]
    force: String,
}
