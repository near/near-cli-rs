use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod call_function;
pub mod deploy;
mod download_contract_abi;
mod download_wasm;
mod inspect_contract;
mod view_storage;

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
/// Choose a contract action:
pub enum ContractActions {
    #[strum_discriminants(strum(
        message = "call-function   - Execute function (contract method)"
    ))]
    /// Execute function (contract method)
    CallFunction(self::call_function::CallFunctionCommands),
    #[strum_discriminants(strum(message = "deploy          - Add a new contract code"))]
    /// Add a contract code
    Deploy(self::deploy::Contract),
    #[strum_discriminants(strum(
        message = "inspect         - Get a list of available function names"
    ))]
    /// Get a list of available function names
    Inspect(self::inspect_contract::Contract),
    #[strum_discriminants(strum(message = "download-abi    - Download contract ABI"))]
    /// Download contract ABI
    DownloadAbi(self::download_contract_abi::Contract),
    #[strum_discriminants(strum(message = "download-wasm   - Download wasm"))]
    /// Download wasm
    DownloadWasm(self::download_wasm::ContractAccount),
    #[strum_discriminants(strum(message = "view-storage    - View contract storage state"))]
    /// View contract storage state
    ViewStorage(self::view_storage::ViewStorage),
}
