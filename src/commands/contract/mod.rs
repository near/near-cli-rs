use crate::common::RpcQueryResponseExt;
use color_eyre::eyre::{Context, Report};
use near_primitives::types::BlockReference;
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
    #[cfg_attr(
        feature = "inspect_contract",
        strum_discriminants(strum(
            message = "inspect          - Get a list of available function names"
        ))
    )]
    #[cfg_attr(
        feature = "inspect_contract",
        doc = "Get a list of available function names"
    )]
    #[cfg(feature = "inspect_contract")]
    Inspect(self::inspect::Contract),
    #[cfg_attr(
        feature = "verify_contract",
        strum_discriminants(strum(
            message = "verify           - Verify the contract for compliance with the program code"
        ))
    )]
    #[cfg_attr(
        feature = "verify_contract",
        doc = "Verify the contract for compliance with the program code"
    )]
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
    json_rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block_reference: &BlockReference,
    account_id: &near_primitives::types::AccountId,
) -> Result<near_abi::AbiRoot, FetchAbiError> {
    let mut retries_left = (0..5).rev();
    loop {
        let contract_abi_response = json_rpc_client
            .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: block_reference.clone(),
                request: near_primitives::views::QueryRequest::CallFunction {
                    account_id: account_id.clone(),
                    method_name: "__contract_abi".to_owned(),
                    args: near_primitives::types::FunctionArgs::from(vec![]),
                },
            })
            .await;

        match contract_abi_response {
            Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(_))
                if retries_left.next().is_some() =>
            {
                eprintln!("Transport error.\nPlease wait. The next try to send this query is happening right now ...");
            }
            Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(
                near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(
                    near_jsonrpc_primitives::types::query::RpcQueryError::ContractExecutionError {
                        vm_error,
                        ..
                    },
                ),
            )) if vm_error.contains("MethodNotFound") => {
                return Err(FetchAbiError::AbiNotSupported);
            }
            Err(err) => {
                return Err(FetchAbiError::RpcError(err));
            }
            Ok(contract_abi_response) => {
                return serde_json::from_slice::<near_abi::AbiRoot>(
                    &zstd::decode_all(
                        contract_abi_response
                            .call_result()
                            .map_err(FetchAbiError::AbiUnknownFormat)?
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
    #[error("Contract does not support NEAR ABI (https://github.com/near/abi), so there is no way to get details about the function argument and return values.")]
    AbiNotSupported,
    #[error("The contract has unknown NEAR ABI format (https://github.com/near/abi), so there is no way to get details about the function argument and return values. See more details about the error:\n\n{0}")]
    AbiUnknownFormat(Report),
    #[error("'__contract_abi' function call failed due to RPC error, so there is no way to get details about the function argument and return values. See more details about the error:\n\n{0}")]
    RpcError(
        near_jsonrpc_client::errors::JsonRpcError<
            near_jsonrpc_primitives::types::query::RpcQueryError,
        >,
    ),
}
