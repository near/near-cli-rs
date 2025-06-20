use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod call_function;
pub mod deploy;
pub mod deploy_global;
mod download_abi;
pub mod download_wasm;
mod inspect;
mod verify;
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
        message = "call-function    - Execute function (contract method)"
    ))]
    /// Execute function (contract method)
    CallFunction(self::call_function::CallFunctionCommands),
    #[strum_discriminants(strum(
        message = "deploy           - Deploy own WASM code or re-use an existing global code from the chain"
    ))]
    /// Add a contract code
    Deploy(self::deploy::Contract),
    #[strum_discriminants(strum(
        message = "deploy-as-global - Deploy a WASM contract code to the global contract code on-chain registry"
    ))]
    /// Add a global contract code
    DeployAsGlobal(self::deploy_global::Contract),
    #[strum_discriminants(strum(
        message = "inspect          - Get a list of available function names"
    ))]
    /// Get a list of available function names
    Inspect(self::inspect::Contract),
    #[strum_discriminants(strum(
        message = "verify           - Verify the contract for compliance with the program code"
    ))]
    /// Verify the contract for compliance with the program code
    Verify(self::verify::Contract),
    #[strum_discriminants(strum(message = "download-abi     - Download contract ABI"))]
    /// Download contract ABI
    DownloadAbi(self::download_abi::Contract),
    #[strum_discriminants(strum(message = "download-wasm    - Download wasm"))]
    /// Download wasm
    DownloadWasm(self::download_wasm::Contract),
    #[strum_discriminants(strum(message = "view-storage     - View contract storage state"))]
    /// View contract storage state
    ViewStorage(self::view_storage::ViewStorage),
}
