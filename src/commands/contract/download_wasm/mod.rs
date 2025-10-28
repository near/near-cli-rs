use std::io::Write;

use clap::ValueEnum;
use color_eyre::eyre::Context;
use inquire::{CustomType, Select};

use crate::common::JsonRpcClientExt;

#[derive(Debug, strum_macros::Display, PartialEq, ValueEnum, Clone, Copy)]
#[strum(serialize_all = "lowercase")]
pub enum ContractType {
    Global,
    Regular,
}

impl interactive_clap::ToCli for ContractType {
    type CliVariant = ContractType;
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ContractContext)]
pub struct Contract {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the contract account ID?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Enter the name of the file to save the contract:
    save_to_file: DownloadContract,
}

#[derive(Debug, Clone)]
pub struct ContractContext {
    global_context: crate::GlobalContext,
    account_id: near_primitives::types::AccountId,
}

impl ContractContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<Contract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            account_id: scope.account_id.clone().into(),
        })
    }
}

impl Contract {
    pub fn input_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is the contract account ID?",
        )
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = ContractContext)]
#[interactive_clap(output_context = DownloadContractContext)]
pub struct DownloadContract {
    #[interactive_clap(skip_default_input_arg)]
    contract_type: Option<ContractType>,
    #[interactive_clap(skip_default_input_arg)]
    /// Enter the name of the file to save the contract:
    file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

impl DownloadContract {
    pub fn input_contract_type(
        _context: &ContractContext,
    ) -> color_eyre::eyre::Result<Option<ContractType>> {
        let selection = Select::new(
            "Which type of contract do you want to download?",
            vec![ContractType::Global, ContractType::Regular],
        )
        .prompt()?;
        Ok(Some(selection))
    }
}

#[derive(Clone)]
pub struct DownloadContractContext(crate::network_view_at_block::ArgsForViewContext);

impl DownloadContractContext {
    pub fn from_previous_context(
        previous_context: ContractContext,
        scope: &<DownloadContract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let contract_type = scope.contract_type.unwrap_or(ContractType::Regular);
            let account_id = previous_context.account_id.clone();
            let file_path: std::path::PathBuf = scope.file_path.clone().into();

            move |network_config, block_reference| {
                download_contract_code(contract_type, &account_id, &file_path, network_config, block_reference.clone())
            }
        });
        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.global_context.config,
            on_after_getting_block_reference_callback,
            interacting_with_account_ids: vec![previous_context.account_id],
        }))
    }
}

impl From<DownloadContractContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: DownloadContractContext) -> Self {
        item.0
    }
}

impl DownloadContract {
    fn input_file_path(
        context: &ContractContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::path_buf::PathBuf>> {
        Ok(Some(
            CustomType::new("Enter the name of the file to save the contract:")
                .with_starting_input(&format!(
                    "{}.wasm",
                    context.account_id.as_str().replace('.', "_")
                ))
                .prompt()?,
        ))
    }
}

fn download_contract_code(
    contract_type: ContractType,
    account_id: &near_primitives::types::AccountId,
    file_path: &std::path::PathBuf,
    network_config: &crate::config::NetworkConfig,
    block_reference: near_primitives::types::BlockReference,
) -> crate::CliResult {
    let code = get_code(contract_type, account_id, network_config, block_reference)?;
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

#[tracing::instrument(name = "Download contract code ...", skip_all)]
pub fn get_code(
    contract_type: ContractType,
    account_id: &near_primitives::types::AccountId,
    network_config: &crate::config::NetworkConfig,
    block_reference: near_primitives::types::BlockReference,
) -> color_eyre::eyre::Result<Vec<u8>> {
    let request = match contract_type {
        ContractType::Global => {
            near_primitives::views::QueryRequest::ViewGlobalContractCodeByAccountId {
                account_id: account_id.clone(),
            }
        }
        ContractType::Regular => near_primitives::views::QueryRequest::ViewCode {
            account_id: account_id.clone(),
        },
    };

    let query_view_method_response = network_config
        .json_rpc_client()
        .blocking_call(near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference,
            request,
        })
        .wrap_err_with(|| {
            format!(
                "Failed to fetch query ViewCode for <{}> on network <{}>",
                account_id, network_config.network_name
            )
        })?;

    let call_access_view =
        if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewCode(result) =
            query_view_method_response.kind
        {
            result
        } else {
            return Err(color_eyre::Report::msg("Error call result".to_string()));
        };
    Ok(call_access_view.code)
}
