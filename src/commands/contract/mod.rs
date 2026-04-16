use color_eyre::eyre::{Context, Report};
use near_kit::BlockReference;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};
use thiserror::Error;

pub mod call_function;
pub mod deploy;
pub mod deploy_global;
mod download_abi;
pub mod download_wasm;
#[cfg(feature = "inspect_contract")]
mod inspect;

#[cfg(feature = "verify_contract")]
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
    #[cfg(feature = "inspect_contract")]
    Inspect(self::inspect::Contract),
    #[strum_discriminants(strum(
        message = "verify           - Verify the contract for compliance with the program code"
    ))]
    /// Verify the contract for compliance with the program code
    #[cfg(feature = "verify_contract")]
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
#[tracing::instrument(name = "Obtaining the ABI for the contract ...", skip_all)]
pub async fn get_contract_abi(
    network_config: &crate::config::NetworkConfig,
    block_reference: &BlockReference,
    account_id: &near_kit::AccountId,
) -> Result<near_abi::AbiRoot, FetchAbiError> {
    tracing::info!(target: "near_teach_me", "Obtaining the ABI for the contract ...");
    let nk_block_ref = block_reference;
    let mut retries_left = (0..5).rev();
    loop {
        let result = network_config
            .client()
            .rpc()
            .view_function(
                account_id,
                "__contract_abi",
                &[],
                nk_block_ref.clone(),
            )
            .await;

        match result {
            Err(ref err) if err.is_retryable() && retries_left.next().is_some() => {
                eprintln!(
                    "Transport error.\nPlease wait. The next try to send this query is happening right now ..."
                );
            }
            Err(near_kit::RpcError::ContractExecution { message, .. })
                if message.contains("MethodNotFound") =>
            {
                return Err(FetchAbiError::AbiNotSupported);
            }
            Err(near_kit::RpcError::FunctionCall { panic, .. })
                if panic.as_deref().unwrap_or("").contains("MethodNotFound") =>
            {
                return Err(FetchAbiError::AbiNotSupported);
            }
            Err(err) => {
                return Err(FetchAbiError::RpcError(err.to_string()));
            }
            Ok(view_function_result) => {
                return serde_json::from_slice::<near_abi::AbiRoot>(
                    &zstd::decode_all(
                        view_function_result
                            .result
                            .as_slice(),
                    )
                    .wrap_err("Failed to 'zstd::decode_all' NEAR ABI")
                    .map_err(FetchAbiError::AbiUnknownFormat)?,
                )
                .wrap_err("Failed to parse NEAR ABI schema")
                .map_err(FetchAbiError::AbiUnknownFormat);
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
#[derive(Error, Debug)]
pub enum FetchAbiError {
    #[error(
        "Contract does not support NEAR ABI (https://github.com/near/abi), so there is no way to get details about the function argument and return values."
    )]
    AbiNotSupported,
    #[error(
        "The contract has unknown NEAR ABI format (https://github.com/near/abi), so there is no way to get details about the function argument and return values. See more details about the error:\n\n{0}"
    )]
    AbiUnknownFormat(Report),
    #[error(
        "'__contract_abi' function call failed due to RPC error, so there is no way to get details about the function argument and return values. See more details about the error:\n\n{0}"
    )]
    RpcError(String),
}
