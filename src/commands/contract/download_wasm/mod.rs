use std::io::Write;

use clap::ValueEnum;
use color_eyre::eyre::Context;
use inquire::CustomType;

use crate::common::JsonRpcClientExt;

#[derive(Debug, strum::Display, PartialEq, ValueEnum, Clone, Copy)]
#[strum(serialize_all = "kebab_case")]
pub enum ContractKind {
    Regular,
    GlobalContractByAccountId,
    GlobalContractByHash,
}

impl interactive_clap::ToCli for ContractKind {
    type CliVariant = ContractKind;
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ContractContext)]
pub struct Contract {
    #[interactive_clap(skip_default_input_arg)]
    /// Which type of contract do you want to download?
    contract_kind: ContractKind,
    #[interactive_clap(named_arg)]
    /// Enter the name of the file to save the contract:
    save_to_file: DownloadContract,
}

impl Contract {
    pub fn input_contract_kind(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<ContractKind>> {
        #[derive(strum_macros::Display, PartialEq)]
        enum Options {
            #[strum(to_string = "Regular contract")]
            Regular,
            #[strum(to_string = "Global contract by account ID")]
            GlobalContractByAccountId,
            #[strum(to_string = "Global contract by hash")]
            GlobalContractByHash,
        }

        impl From<Options> for ContractKind {
            fn from(option: Options) -> Self {
                match option {
                    Options::Regular => ContractKind::Regular,
                    Options::GlobalContractByAccountId => ContractKind::GlobalContractByAccountId,
                    Options::GlobalContractByHash => ContractKind::GlobalContractByHash,
                }
            }
        }

        let selection = inquire::Select::new(
            "Which type of contract do you want to download?",
            vec![
                Options::Regular,
                Options::GlobalContractByAccountId,
                Options::GlobalContractByHash,
            ],
        )
        .prompt()?;
        Ok(Some(selection.into()))
    }
}

#[derive(Debug, Clone)]
pub struct ContractContext {
    global_context: crate::GlobalContext,
    contract_kind: ContractKind,
}

impl ContractContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<Contract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            contract_kind: scope.contract_kind,
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
    /// Account ID (regular/global-contract-by-account-id) or code hash (global-by-hash)
    #[interactive_clap(skip_default_input_arg)]
    target: String,
    #[interactive_clap(skip_default_input_arg)]
    /// Enter the name of the file to save the contract:
    file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

impl DownloadContract {
    pub fn input_target(context: &ContractContext) -> color_eyre::eyre::Result<Option<String>> {
        use inquire::CustomType;

        let target = match context.contract_kind {
            ContractKind::Regular => {
                let Some(account_id) =
                    crate::common::input_non_signer_account_id_from_used_account_list(
                        &context.global_context.config.credentials_home_dir,
                        "What is the contract account ID?",
                    )?
                else {
                    return Ok(None);
                };
                account_id.to_string()
            }
            ContractKind::GlobalContractByAccountId => {
                let Some(account_id) =
                    crate::common::input_non_signer_account_id_from_used_account_list(
                        &context.global_context.config.credentials_home_dir,
                        "What is the global contract account ID?",
                    )?
                else {
                    return Ok(None);
                };
                account_id.to_string()
            }
            ContractKind::GlobalContractByHash => {
                CustomType::<near_primitives::hash::CryptoHash>::new(
                    "What is the global contract code hash?",
                )
                .prompt()?
                .to_string()
            }
        };

        Ok(Some(target))
    }
}

#[derive(Clone)]
pub struct DownloadContractContext {
    pub args: crate::network_view_at_block::ArgsForViewContext,
    pub file_path: std::path::PathBuf,
}

impl DownloadContractContext {
    pub fn from_previous_context(
        previous_context: ContractContext,
        scope: &<DownloadContract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let contract_kind = previous_context.contract_kind;
            let target = scope.target.clone();
            let file_path: std::path::PathBuf = scope.file_path.clone().into();

            move |network_config, block_reference| {
                download_contract_code(contract_kind, &target, &file_path, network_config, block_reference.clone())
            }
        });

        let interacting_with_account_ids = match previous_context.contract_kind {
            ContractKind::Regular => vec![scope.target.parse()?],
            ContractKind::GlobalContractByAccountId => vec![scope.target.parse()?],
            ContractKind::GlobalContractByHash => vec![],
        };

        Ok(Self {
            args: crate::network_view_at_block::ArgsForViewContext {
                config: previous_context.global_context.config,
                on_after_getting_block_reference_callback,
                interacting_with_account_ids,
            },
            file_path: scope.file_path.clone().into(),
        })
    }
}

impl From<DownloadContractContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: DownloadContractContext) -> Self {
        item.args
    }
}

impl DownloadContract {
    fn input_file_path(
        _context: &ContractContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::path_buf::PathBuf>> {
        Ok(Some(
            CustomType::new("Enter the name of the file to save the contract:")
                .with_starting_input("config.wasm")
                .prompt()?,
        ))
    }
}

fn download_contract_code(
    contract_kind: ContractKind,
    target: &str,
    file_path: &std::path::PathBuf,
    network_config: &crate::config::NetworkConfig,
    block_reference: near_primitives::types::BlockReference,
) -> crate::CliResult {
    let code = get_code(contract_kind, target, network_config, block_reference)?;
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
    contract_kind: ContractKind,
    target: &str,
    network_config: &crate::config::NetworkConfig,
    block_reference: near_primitives::types::BlockReference,
) -> color_eyre::eyre::Result<Vec<u8>> {
    let request = match contract_kind {
        ContractKind::Regular => near_primitives::views::QueryRequest::ViewCode {
            account_id: target.parse()?,
        },
        ContractKind::GlobalContractByAccountId => {
            near_primitives::views::QueryRequest::ViewGlobalContractCodeByAccountId {
                account_id: target.parse()?,
            }
        }
        ContractKind::GlobalContractByHash => {
            near_primitives::views::QueryRequest::ViewGlobalContractCode {
                code_hash: target.parse().map_err(|e| {
                    color_eyre::eyre::eyre!("Failed to parse code hash <{}>: {}", target, e)
                })?,
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
            target, block_height, network_config.network_name
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
        target,
        network_config.network_name,
        block_height + number_of_shards * 2 - block_height + 1
    )))
}
