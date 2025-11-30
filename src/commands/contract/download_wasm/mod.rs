use std::io::Write;

use color_eyre::eyre::Context;
use inquire::CustomType;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

use crate::common::JsonRpcClientExt;

#[derive(Debug, Clone)]
pub enum ContractType {
    Regular(near_primitives::types::AccountId),
    GlobalContractByAccountId(near_primitives::types::AccountId),
    GlobalContractByCodeHash(near_primitives::hash::CryptoHash),
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
            ContractType::GlobalContractByCodeHash(code_hash) => {
                write!(f, "global-contract-by-hash:{}", code_hash)
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
    GlobalContractByCodeHash(DownloadGlobalContractByCodeHash),
}

#[derive(Clone)]
pub struct ArgsForDownloadContract {
    pub config: crate::config::Config,
    pub contract_type: ContractType,
    pub interacting_with_account_ids: Vec<near_primitives::types::AccountId>,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = ArgsForDownloadContract)]
#[interactive_clap(output_context = DownloadContractContext)]
pub struct DownloadContract {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter the name of the file to save the contract:
    file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

impl DownloadContract {
    fn input_file_path(
        _context: &ArgsForDownloadContract,
    ) -> color_eyre::eyre::Result<Option<crate::types::path_buf::PathBuf>> {
        Ok(Some(
            CustomType::new("Enter the name of the file to save the contract:")
                .with_starting_input("contract.wasm")
                .prompt()?,
        ))
    }
}

pub struct DownloadContractContext(crate::network_view_at_block::ArgsForViewContext);

impl DownloadContractContext {
    pub fn from_previous_context(
        previous_context: ArgsForDownloadContract,
        scope: &<DownloadContract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let config = previous_context.config;
        let contract_type = previous_context.contract_type.clone();
        let interacting_with_account_ids = previous_context.interacting_with_account_ids.clone();
        let file_path: std::path::PathBuf = scope.file_path.clone().into();

        let on_after_getting_block_reference_callback:
            crate::network_view_at_block::OnAfterGettingBlockReferenceCallback =
                std::sync::Arc::new(move |network_config, block_reference| {
                    download_contract_code(
                        &contract_type,
                        &file_path,
                        network_config,
                        block_reference.clone(),
                    )
                });

        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config,
            on_after_getting_block_reference_callback,
            interacting_with_account_ids,
        }))
    }
}

impl From<DownloadContractContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: DownloadContractContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = DownloadRegularContractContext)]
pub struct DownloadRegularContract {
    #[interactive_clap(skip_default_input_arg)]
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    save_to_file: DownloadContract,
}

impl DownloadRegularContract {
    pub fn input_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is the contract account ID?",
        )
    }
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

impl From<DownloadRegularContractContext> for ArgsForDownloadContract {
    fn from(context: DownloadRegularContractContext) -> Self {
        let account_id: near_primitives::types::AccountId = context.account_id.clone().into();

        Self {
            config: context.global_context.config,
            contract_type: ContractType::Regular(account_id.clone()),
            interacting_with_account_ids: vec![account_id],
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = DownloadGlobalContractByAccountIdContext)]
pub struct DownloadGlobalContractByAccountId {
    #[interactive_clap(skip_default_input_arg)]
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    save_to_file: DownloadContract,
}

impl DownloadGlobalContractByAccountId {
    pub fn input_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is the account ID of the global contract?",
        )
    }
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

impl From<DownloadGlobalContractByAccountIdContext> for ArgsForDownloadContract {
    fn from(context: DownloadGlobalContractByAccountIdContext) -> Self {
        let account_id: near_primitives::types::AccountId = context.account_id.clone().into();

        Self {
            config: context.global_context.config,
            contract_type: ContractType::GlobalContractByAccountId(account_id.clone()),
            interacting_with_account_ids: vec![account_id],
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = DownloadGlobalContractByCodeHashContext)]
pub struct DownloadGlobalContractByCodeHash {
    /// What is the contract hash of the global contract?
    contrach_hash: crate::types::crypto_hash::CryptoHash,
    #[interactive_clap(named_arg)]
    save_to_file: DownloadContract,
}

#[derive(Clone)]
pub struct DownloadGlobalContractByCodeHashContext {
    global_context: crate::GlobalContext,
    code_hash: crate::types::crypto_hash::CryptoHash,
}

impl DownloadGlobalContractByCodeHashContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<DownloadGlobalContractByCodeHash as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            code_hash: scope.contrach_hash,
        })
    }
}

impl From<DownloadGlobalContractByCodeHashContext> for ArgsForDownloadContract {
    fn from(context: DownloadGlobalContractByCodeHashContext) -> Self {
        let code_hash: near_primitives::hash::CryptoHash = context.code_hash.into();

        Self {
            config: context.global_context.config,
            contract_type: ContractType::GlobalContractByCodeHash(code_hash),
            interacting_with_account_ids: vec![],
        }
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
        ContractType::GlobalContractByCodeHash(code_hash) => {
            near_primitives::views::QueryRequest::ViewGlobalContractCode { code_hash }
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
