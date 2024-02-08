use color_eyre::{eyre::Context, owo_colors::OwoColorize};
use std::fmt::Write;

use near_primitives::types::{BlockId, BlockReference};

use crate::common::{CallResultExt, JsonRpcClientExt, RpcQueryResponseExt};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ContractContext)]
pub struct Contract {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the contract account ID?
    contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

impl Contract {
    pub fn input_contract_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is the contract account ID?",
        )
    }
}

#[derive(Clone)]
pub struct ContractContext(crate::network_view_at_block::ArgsForViewContext);

impl ContractContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<Contract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let account_id: near_primitives::types::AccountId = scope.contract_account_id.clone().into();
            move |network_config, block_reference| {
                let view_code_response = network_config
                    .json_rpc_client()
                    .blocking_call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                        block_reference: block_reference.clone(),
                        request: near_primitives::views::QueryRequest::ViewCode {
                            account_id: account_id.clone(),
                        },
                    })
                    .wrap_err_with(|| format!("Failed to fetch query ViewCode for <{}> on network <{}>", &account_id, network_config.network_name))?;

                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(display_inspect_contract(&account_id, network_config, view_code_response))
            }
        });
        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.config,
            on_after_getting_block_reference_callback,
            interacting_with_account_ids: vec![scope.contract_account_id.clone().into()],
        }))
    }
}

impl From<ContractContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: ContractContext) -> Self {
        item.0
    }
}

async fn display_inspect_contract(
    account_id: &near_primitives::types::AccountId,
    network_config: &crate::config::NetworkConfig,
    view_code_response: near_jsonrpc_primitives::types::query::RpcQueryResponse,
) -> crate::CliResult {
    let json_rpc_client = network_config.json_rpc_client();
    let block_reference = BlockReference::from(BlockId::Hash(view_code_response.block_hash));
    let contract_code_view =
        if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewCode(result) =
            view_code_response.kind
        {
            result
        } else {
            return Err(color_eyre::Report::msg("Error call result".to_string()));
        };

    let account_view = get_account_view(
        network_config,
        &json_rpc_client,
        &block_reference,
        account_id,
    )
    .await?;

    let access_keys = get_access_keys(
        network_config,
        &json_rpc_client,
        &block_reference,
        account_id,
    )
    .await?;

    let mut table = prettytable::Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_NO_COLSEP);

    table.add_row(prettytable::row![
            Fg->account_id,
            format!("At block #{}\n({})", view_code_response.block_height, view_code_response.block_hash)
        ]);

    let contract_status = if account_view.code_hash == near_primitives::hash::CryptoHash::default()
    {
        "No contract code".to_string()
    } else {
        hex::encode(account_view.code_hash.as_ref())
    };
    table.add_row(prettytable::row![
        Fy->"SHA-256 checksum hex",
        contract_status
    ]);

    table.add_row(prettytable::row![
        Fy->"Storage used",
        format!("{} ({} Wasm + {} data)",
            bytesize::ByteSize(account_view.storage_usage),
            bytesize::ByteSize(u64::try_from(contract_code_view.code.len())?),
            bytesize::ByteSize(
                account_view.storage_usage
                    .checked_sub(u64::try_from(contract_code_view.code.len())?)
                    .expect("Unexpected error")
            )
        )
    ]);

    let access_keys_summary = if access_keys.is_empty() {
        "Contract is locked (no access keys)".to_string()
    } else {
        let full_access_keys_count = access_keys
            .iter()
            .filter(|access_key| {
                matches!(
                    access_key.access_key.permission,
                    near_primitives::views::AccessKeyPermissionView::FullAccess
                )
            })
            .count();
        format!(
            "{} full access keys and {} function-call-only access keys",
            full_access_keys_count,
            access_keys.len() - full_access_keys_count
        )
    };
    table.add_row(prettytable::row![
        Fy->"Access keys",
        access_keys_summary
    ]);

    if let Ok(contract_source_metadata_response) = get_contract_source_metadata(
        network_config,
        &json_rpc_client,
        &block_reference,
        account_id,
    )
    .await
    {
        if let Ok(contract_source_metadata) = contract_source_metadata_response
            .call_result()?
            .parse_result_from_json::<near_contract_standards::contract_metadata::ContractSourceMetadata>()
            {
                table.add_row(prettytable::row![
                    Fy->"Contract version",
                    contract_source_metadata.version.unwrap_or_default()
                ]);
                table.add_row(prettytable::row![
                    Fy->"Contract link",
                    contract_source_metadata.link.unwrap_or_default()
                ]);
                table.add_row(prettytable::row![
                    Fy->"Supported standards",
                    contract_source_metadata.standards
                        .iter()
                        .fold(String::new(), |mut output, standard| {
                            let _ = writeln!(output, "{} ({})", standard.standard, standard.version);
                            output
                        })
                ]);
            }

        if let Ok(contract_abi_response) = get_contract_abi(
            network_config,
            &json_rpc_client,
            &block_reference,
            account_id,
        )
        .await
        {
            let call_result = contract_abi_response.call_result()?;
            if let Ok(abi_root) = serde_json::from_slice::<near_abi::AbiRoot>(&zstd::decode_all(
                &call_result.result[..],
            )?) {
                table.add_row(prettytable::row![
                    Fy->"Schema version",
                    abi_root.schema_version
                ]);
                table.add_empty_row();
                table.add_row(prettytable::row![Fy->"Metods:"]);
                for function in abi_root.body.functions {
                    table.add_row(prettytable::row![
                        Fg->function.name,
                        function.doc.unwrap_or_default()
                    ]);
                    table.add_row(prettytable::row![
                        "",
                        Fy->"Kind of function call",
                        serde_json::to_string(&function.kind).unwrap_or_default()
                    ]);
                    table.add_row(prettytable::row![
                        "",
                        Fy->"Modifiers",
                        serde_json::to_string(&function.modifiers).unwrap_or_default()
                    ]);
                    if !function.params.is_empty() {
                        match function.params {
                            near_abi::AbiParameters::Borsh { .. } => {
                                panic!("Borsh is currently unsupported")
                            }
                            near_abi::AbiParameters::Json { args } => {
                                table.add_row(prettytable::row![
                                    "",
                                    Fy->"Arguments",
                                    args.iter()
                                        .fold(String::new(), |mut output, arg| {
                                            let _ = writeln!(output, "{} ({})",
                                                arg.name,
                                                if let Some(reference) = arg.type_schema.clone().into_object().reference {
                                                    reference
                                                } else if let Some(instance_type) = arg.type_schema.clone().into_object().instance_type {
                                                    serde_json::to_string(&instance_type).unwrap_or_default()
                                                } else {
                                                    "".to_string()
                                                });
                                            output
                                        })
                                ]);
                            }
                        }
                    }
                    table.add_row(prettytable::row![
                        "",
                        Fy->"Return type identifier",
                        match function.result {
                            Some(r_type) => {
                                match r_type {
                                    near_abi::AbiType::Borsh { type_schema: _ } => panic!("Borsh is currently unsupported"),
                                    near_abi::AbiType::Json { type_schema } => {
                                        serde_json::to_string(&type_schema.into_object().instance_type).unwrap_or_default()
                                    },
                                    }
                            },
                            None => "".to_string()
                        }
                    ]);
                }
            }
        }
        table.printstd();
        return Ok(());
    }

    table.add_empty_row();
    table.add_row(prettytable::row![
        Fy->"Metods:",
        format!(
            "Contact does not support ABI ({}),\nso there is no way to get detailed information",
            "https://github.com/near/abi".yellow()
        )
    ]);

    for function in wasmer::Module::from_binary(&wasmer::Store::default(), &contract_code_view.code)
            .wrap_err_with(|| format!("Could not create new WebAssembly module from Wasm binary for contract <{account_id}>."))?
            .exports()
        {
            table.add_row(prettytable::row![Fg->function.name()]);
        }
    table.printstd();
    Ok(())
}

async fn get_account_view(
    network_config: &crate::config::NetworkConfig,
    json_rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block_reference: &BlockReference,
    account_id: &near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<near_primitives::views::AccountView> {
    for _ in 0..5 {
        let account_view_response = json_rpc_client
            .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: block_reference.clone(),
                request: near_primitives::views::QueryRequest::ViewAccount {
                    account_id: account_id.clone(),
                },
            })
            .await;

        if let Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(_)) =
            &account_view_response
        {
            eprintln!("Transport error.\nPlease wait. The next try to send this query is happening right now ...");
            std::thread::sleep(std::time::Duration::from_millis(100))
        } else {
            return account_view_response
                .wrap_err_with(|| {
                    format!(
                        "Failed to fetch query ViewAccount for contract <{}> on network <{}>",
                        account_id, network_config.network_name
                    )
                })?
                .account_view();
        }
    }
    color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(format!(
        "Transport error. Failed to fetch query ViewAccount for contract <{}> on network <{}>",
        account_id, network_config.network_name
    )))
}

async fn get_access_keys(
    network_config: &crate::config::NetworkConfig,
    json_rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block_reference: &BlockReference,
    account_id: &near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<Vec<near_primitives::views::AccessKeyInfoView>> {
    for _ in 0..5 {
        let access_keys_response = json_rpc_client
            .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: block_reference.clone(),
                request: near_primitives::views::QueryRequest::ViewAccessKeyList {
                    account_id: account_id.clone(),
                },
            })
            .await;

        if let Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(_)) =
            &access_keys_response
        {
            eprintln!("Transport error.\nPlease wait. The next try to send this query is happening right now ...");
            std::thread::sleep(std::time::Duration::from_millis(100))
        } else {
            return Ok(access_keys_response
                .wrap_err_with(|| {
                    format!(
                        "Failed to fetch ViewAccessKeyList for contract <{}> on network <{}>",
                        account_id, network_config.network_name
                    )
                })?
                .access_key_list_view()?
                .keys);
        }
    }
    color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(format!(
        "Transport error. Failed to fetch query ViewAccessKeyList for contract <{}> on network <{}>",
        account_id, network_config.network_name
    )))
}

async fn get_contract_source_metadata(
    network_config: &crate::config::NetworkConfig,
    json_rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block_reference: &BlockReference,
    account_id: &near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<near_jsonrpc_primitives::types::query::RpcQueryResponse> {
    for _ in 0..5 {
        let contract_source_metadata_response = json_rpc_client
            .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: block_reference.clone(),
                request: near_primitives::views::QueryRequest::CallFunction {
                    account_id: account_id.clone(),
                    method_name: "contract_source_metadata".to_owned(),
                    args: near_primitives::types::FunctionArgs::from(vec![]),
                },
            })
            .await;

        if let Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(_)) =
            &contract_source_metadata_response
        {
            eprintln!("Transport error.\nPlease wait. The next try to send this query is happening right now ...");
            std::thread::sleep(std::time::Duration::from_millis(100))
        } else {
            return contract_source_metadata_response.wrap_err_with(|| {
                format!(
                    "Failed to fetch 'contract_source_metadata' for account <{}> on network <{}>",
                    account_id, network_config.network_name
                )
            });
        }
    }
    color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(format!(
        "Transport error. Failed to fetch 'contract_source_metadata' for account <{}> on network <{}>",
        account_id, network_config.network_name
    )))
}

async fn get_contract_abi(
    network_config: &crate::config::NetworkConfig,
    json_rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block_reference: &BlockReference,
    account_id: &near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<near_jsonrpc_primitives::types::query::RpcQueryResponse> {
    for _ in 0..5 {
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

        if let Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(_)) =
            &contract_abi_response
        {
            eprintln!("Transport error.\nPlease wait. The next try to send this query is happening right now ...");
            std::thread::sleep(std::time::Duration::from_millis(100))
        } else {
            return contract_abi_response.wrap_err_with(|| {
                format!(
                    "Failed to fetch 'contract_abi' for account <{}> on network <{}>",
                    account_id, network_config.network_name
                )
            });
        }
    }
    color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(format!(
        "Transport error. Failed to fetch 'contract_abi' for account <{}> on network <{}>",
        account_id, network_config.network_name
    )))
}
