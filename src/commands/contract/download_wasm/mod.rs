use std::io::Write;

use color_eyre::eyre::Context;
use inquire::CustomType;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

use crate::common::JsonRpcClientExt;

#[derive(Debug, Clone)]
pub enum ContractType {
    Regular(near_primitives::types::AccountId),
    GlobalContractByAccountId(near_primitives::types::AccountId),
    GlobalContractByContractHash(near_primitives::hash::CryptoHash),
}

impl std::fmt::Display for ContractType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContractType::Regular(account_id) => {
                write!(f, "regular:{}", account_id)
            }
            ContractType::GlobalContractByAccountId(account_id) => {
                write!(f, "global-contract-by-account-id:{}", account_id)
            }
            ContractType::GlobalContractByContractHash(contract_hash) => {
                write!(f, "global-contract-by-hash:{}", contract_hash)
            }
        }
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// Which type of contract do you want to pass?
pub enum ContractKind {
    #[strum_discriminants(strum(
        message = "Regular                       - Regular contract deployed to an account"
    ))]
    Regular(DownloadRegularContract),
    #[strum_discriminants(strum(
        message = "Global contract by account id - Global contract identified by account ID"
    ))]
    GlobalContractByAccountId(DownloadGlobalContractByAccountId),
    #[strum_discriminants(strum(
        message = "Global contract by hash       - Global contract identified by code hash"
    ))]
    GlobalContractByContractHash(DownloadGlobalContractByContractHash),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = DownloadRegularContractContext)]
pub struct DownloadRegularContract {
    /// What is the contract account ID?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    save_to_file: DownloadRegularContractAction,
}

#[derive(Clone)]
pub struct DownloadRegularContractContext {
    global_context: crate::GlobalContext,
    account_id: crate::types::account_id::AccountId,
}

impl DownloadRegularContractContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<DownloadRegularContract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            account_id: scope.account_id.clone(),
        })
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = DownloadRegularContractContext)]
#[interactive_clap(output_context = DownloadRegularContractActionContext)]
pub struct DownloadRegularContractAction {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter the name of the file to save the contract:
    file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct DownloadRegularContractActionContext(crate::network_view_at_block::ArgsForViewContext);

impl DownloadRegularContractActionContext {
    pub fn from_previous_context(
        previous_context: DownloadRegularContractContext,
        scope: &<DownloadRegularContractAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let contract_type = ContractType::Regular(previous_context.account_id.clone().into());
            let file_path: std::path::PathBuf = scope.file_path.clone().into();

            move |network_config, block_reference| {
                download_contract_code(&contract_type, &file_path, network_config, block_reference.clone())
            }
        });

        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.global_context.config,
            on_after_getting_block_reference_callback,
            interacting_with_account_ids: vec![previous_context.account_id.into()],
        }))
    }
}

impl From<DownloadRegularContractActionContext>
    for crate::network_view_at_block::ArgsForViewContext
{
    fn from(item: DownloadRegularContractActionContext) -> Self {
        item.0
    }
}

impl DownloadRegularContractAction {
    fn input_file_path(
        _context: &DownloadRegularContractContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::path_buf::PathBuf>> {
        Ok(Some(
            CustomType::new("Enter the name of the file to save the contract:")
                .with_starting_input("contract.wasm")
                .prompt()?,
        ))
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = DownloadGlobalContractByAccountIdContext)]
pub struct DownloadGlobalContractByAccountId {
    /// What is the account ID of the global contract?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    save_to_file: DownloadGlobalContractByAccountIdAction,
}

#[derive(Clone)]
pub struct DownloadGlobalContractByAccountIdContext {
    global_context: crate::GlobalContext,
    account_id: crate::types::account_id::AccountId,
}

impl DownloadGlobalContractByAccountIdContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<DownloadGlobalContractByAccountId as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            account_id: scope.account_id.clone(),
        })
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = DownloadGlobalContractByAccountIdContext)]
#[interactive_clap(output_context = DownloadGlobalContractByAccountIdActionContext)]
pub struct DownloadGlobalContractByAccountIdAction {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter the name of the file to save the contract:
    file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct DownloadGlobalContractByAccountIdActionContext(
    crate::network_view_at_block::ArgsForViewContext,
);

impl DownloadGlobalContractByAccountIdActionContext {
    pub fn from_previous_context(
        previous_context: DownloadGlobalContractByAccountIdContext,
        scope: &<DownloadGlobalContractByAccountIdAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let contract_type = ContractType::GlobalContractByAccountId(previous_context.account_id.clone().into());
            let file_path: std::path::PathBuf = scope.file_path.clone().into();

            move |network_config, block_reference| {
                download_contract_code(&contract_type, &file_path, network_config, block_reference.clone())
            }
        });

        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.global_context.config,
            on_after_getting_block_reference_callback,
            interacting_with_account_ids: vec![previous_context.account_id.into()],
        }))
    }
}

impl From<DownloadGlobalContractByAccountIdActionContext>
    for crate::network_view_at_block::ArgsForViewContext
{
    fn from(item: DownloadGlobalContractByAccountIdActionContext) -> Self {
        item.0
    }
}

impl DownloadGlobalContractByAccountIdAction {
    fn input_file_path(
        _context: &DownloadGlobalContractByAccountIdContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::path_buf::PathBuf>> {
        Ok(Some(
            CustomType::new("Enter the name of the file to save the contract:")
                .with_starting_input("contract.wasm")
                .prompt()?,
        ))
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = DownloadGlobalContractByContractHashContext)]
pub struct DownloadGlobalContractByContractHash {
    /// What is the contract hash of the global contract?
    contrach_hash: crate::types::crypto_hash::CryptoHash,
    #[interactive_clap(named_arg)]
    save_to_file: DownloadGlobalContractByContractHashAction,
}

#[derive(Clone)]
pub struct DownloadGlobalContractByContractHashContext {
    global_context: crate::GlobalContext,
    contrach_hash: crate::types::crypto_hash::CryptoHash,
}

impl DownloadGlobalContractByContractHashContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<DownloadGlobalContractByContractHash as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            contrach_hash: scope.contrach_hash,
        })
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = DownloadGlobalContractByContractHashContext)]
#[interactive_clap(output_context = DownloadGlobalContractByContractHashActionContext)]
pub struct DownloadGlobalContractByContractHashAction {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter the name of the file to save the contract:
    file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct DownloadGlobalContractByContractHashActionContext(
    crate::network_view_at_block::ArgsForViewContext,
);

impl DownloadGlobalContractByContractHashActionContext {
    pub fn from_previous_context(
        previous_context: DownloadGlobalContractByContractHashContext,
        scope: &<DownloadGlobalContractByContractHashAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let contract_type = ContractType::GlobalContractByContractHash(previous_context.contrach_hash.into());
            let file_path: std::path::PathBuf = scope.file_path.clone().into();

            move |network_config, block_reference| {
                download_contract_code(&contract_type, &file_path, network_config, block_reference.clone())
            }
        });

        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.global_context.config,
            on_after_getting_block_reference_callback,
            interacting_with_account_ids: vec![],
        }))
    }
}

impl From<DownloadGlobalContractByContractHashActionContext>
    for crate::network_view_at_block::ArgsForViewContext
{
    fn from(item: DownloadGlobalContractByContractHashActionContext) -> Self {
        item.0
    }
}

impl DownloadGlobalContractByContractHashAction {
    fn input_file_path(
        _context: &DownloadGlobalContractByContractHashContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::path_buf::PathBuf>> {
        Ok(Some(
            CustomType::new("Enter the name of the file to save the contract:")
                .with_starting_input("contract.wasm")
                .prompt()?,
        ))
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct Contract {
    #[interactive_clap(subcommand)]
    /// Which type of contract do you want to download?
    contract_kind: ContractKind,
}

fn download_contract_code(
    contract_type: &ContractType,
    file_path: &std::path::PathBuf,
    network_config: &crate::config::NetworkConfig,
    block_reference: near_primitives::types::BlockReference,
) -> crate::CliResult {
    let code = get_code(contract_type, network_config, block_reference)?;
    std::fs::File::create(file_path)
        .wrap_err_with(|| format!("Failed to create file: {file_path:?}"))?
        .write(&code)
        .wrap_err_with(|| format!("Failed to write to file: {file_path:?}"))?;
    tracing::info!(
        parent: &tracing::Span::none(),
        "The file {:?} was downloaded successfully",
        file_path
    );
    Ok(())
}

#[tracing::instrument(name = "Trying to download contract code ...", skip_all)]
pub fn get_code(
    contract_type: &ContractType,
    network_config: &crate::config::NetworkConfig,
    block_reference: near_primitives::types::BlockReference,
) -> color_eyre::eyre::Result<Vec<u8>> {
    let request = match contract_type.clone() {
        ContractType::Regular(account_id) => {
            near_primitives::views::QueryRequest::ViewCode { account_id }
        }
        ContractType::GlobalContractByAccountId(account_id) => {
            near_primitives::views::QueryRequest::ViewGlobalContractCodeByAccountId { account_id }
        }
        ContractType::GlobalContractByContractHash(contract_hash) => {
            near_primitives::views::QueryRequest::ViewGlobalContractCode {
                code_hash: contract_hash,
            }
        }
    };

    let block_height = network_config
        .json_rpc_client()
        .blocking_call(near_jsonrpc_client::methods::block::RpcBlockRequest {
            block_reference: block_reference.clone(),
        })
        .wrap_err_with(|| {
            format!(
                "Failed to fetch block info for block reference {:?} on network <{}>",
                block_reference, network_config.network_name
            )
        })?
        .header
        .height;

    let number_of_shards = network_config
        .json_rpc_client()
        .blocking_call(
            near_jsonrpc_client::methods::EXPERIMENTAL_protocol_config::RpcProtocolConfigRequest {
                block_reference: block_reference.clone(),
            },
        )
        .wrap_err_with(|| {
            format!(
                "Failed to fetch shards info for block height {} on network <{}>",
                block_height, network_config.network_name
            )
        })?
        .shard_layout
        .num_shards();

    for block_height in block_height..=block_height + number_of_shards * 2 {
        tracing::info!(
        parent: &tracing::Span::none(),
            "Trying to fetch contract code for <{}> at block height {} on network <{}>...",
            contract_type, block_height, network_config.network_name
        );

        let Ok(query_view_method_response) = network_config.json_rpc_client().blocking_call(
            near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: near_primitives::types::BlockReference::BlockId(
                    near_primitives::types::BlockId::Height(block_height),
                ),
                request: request.clone(),
            },
        ) else {
            continue;
        };

        let call_access_view =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewCode(result) =
                query_view_method_response.kind
            {
                result
            } else {
                return Err(color_eyre::Report::msg("Error call result".to_string()));
            };

        return Ok(call_access_view.code);
    }

    Err(color_eyre::Report::msg(format!(
        "Failed to fetch contract code for <{}> on network <{}> after trying {} block heights.",
        contract_type,
        network_config.network_name,
        block_height + number_of_shards * 2 - block_height + 1
    )))
}
