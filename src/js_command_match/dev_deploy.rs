#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `dev-deploy` command. Once you run it with the specified arguments, new syntax command will be suggested.
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
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}
