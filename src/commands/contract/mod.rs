use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod call_function;
mod deploy;
mod download_wasm;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct ContractCommands {
    #[interactive_clap(subcommand)]
    contract_actions: ContractActions,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// Сhoose action for account
pub enum ContractActions {
    #[strum_discriminants(strum(
        message = "call-function   - Execute function (contract method)"
    ))]
    /// Execute function (contract method)
    CallFunction(self::call_function::CallFunctionCommands),
    #[strum_discriminants(strum(message = "deploy          - Add a new contract code"))]
    /// Add a contract code
    Deploy(self::deploy::Contract),
    #[strum_discriminants(strum(message = "download-wasm   - Download wasm"))]
    /// Download wasm
    DownloadWasm(self::download_wasm::ContractAccount),
}
