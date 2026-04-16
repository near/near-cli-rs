use base64::Engine as _;
use std::fmt::Write;

use color_eyre::{
    eyre::{Context, Report},
    owo_colors::OwoColorize,
};
use thiserror::Error;
use tracing_indicatif::span_ext::IndicatifSpanExt;

use near_kit::BlockReference;

use super::FetchAbiError;
use crate::common::{
    CallResultExt, RpcResultExt, query_view_access_key_list, query_view_account,
    query_view_function, sleep_after_error,
};

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
            let account_id: near_kit::AccountId = scope.contract_account_id.clone().into();

            move |network_config, block_reference| {
                crate::common::block_on(display_inspect_contract(
                        &account_id,
                        network_config,
                        block_reference,
                    ))
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

#[tracing::instrument(name = "Obtaining the contract code ...", skip_all)]
fn get_contract_code(
    account_id: &near_kit::AccountId,
    network_config: &crate::config::NetworkConfig,
    block_reference: &near_kit::BlockReference,
) -> color_eyre::eyre::Result<serde_json::Value> {
    tracing::info!(target: "near_teach_me", "Obtaining the contract code ...");
    let mut params = serde_json::json!({
        "request_type": "view_code",
        "account_id": account_id.to_string(),
    });
    if let serde_json::Value::Object(block_params) = block_reference.to_rpc_params()
        && let serde_json::Value::Object(map) = &mut params
    {
        map.extend(block_params);
    }
    crate::common::block_on(
        network_config
            .client()
            .rpc()
            .call::<_, serde_json::Value>("query", params),
    )
    .into_eyre()
    .wrap_err_with(|| {
        format!(
            "Failed to fetch query ViewCode for <{}> on network <{}>",
            &account_id, network_config.network_name
        )
    })
}

#[tracing::instrument(name = "Contract inspection ...", skip_all)]
async fn display_inspect_contract(
    account_id: &near_kit::AccountId,
    network_config: &crate::config::NetworkConfig,
    block_reference: &near_kit::BlockReference,
) -> crate::CliResult {
    tracing::info!(target: "near_teach_me", "Contract inspection ...");
    let view_code_json = get_contract_code(account_id, network_config, block_reference)?;
    let code_base64 = view_code_json
        .get("code_base64")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let code_bytes = base64::engine::general_purpose::STANDARD
        .decode(code_base64)
        .unwrap_or_default();
    let block_height = view_code_json
        .get("block_height")
        .and_then(|v| v.as_u64())
        .unwrap_or_default();
    let block_hash_str = view_code_json
        .get("block_hash")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    let account_view = get_account_view(
        &network_config.network_name,
        network_config,
        block_reference,
        account_id,
    )
    .await?;

    let access_keys = get_access_keys(
        &network_config.network_name,
        network_config,
        block_reference,
        account_id,
    )
    .await?;

    let mut table = prettytable::Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_NO_COLSEP);

    table.add_row(prettytable::row![
        Fg->account_id,
        format!("At block #{}\n({})", block_height, block_hash_str)
    ]);

    let (contract_type, contract_status, checksum_hex, checksum_base58, storage_used) = match (
        &account_view.code_hash,
        &account_view.global_contract_account_id,
        &account_view.global_contract_hash,
    ) {
        (_, Some(global_contract_account_id), None) => (
            "Global Contract",
            format!("deployed by account ID <{global_contract_account_id}>"),
            "no data".to_string(),
            "no data".to_string(),
            format!("{}", bytesize::ByteSize(account_view.storage_usage)),
        ),
        (_, None, Some(global_contract_hash)) => (
            "Global Contract",
            "deployed by Hash".to_string(),
            hex::encode(global_contract_hash.as_ref()),
            bs58::encode(global_contract_hash).into_string(),
            format!("{}", bytesize::ByteSize(account_view.storage_usage)),
        ),
        (code_hash, None, None) => (
            "Local Contract",
            String::new(),
            hex::encode(code_hash.as_ref()),
            bs58::encode(code_hash).into_string(),
            format!(
                "{} ({} Wasm + {} data)",
                bytesize::ByteSize(account_view.storage_usage),
                bytesize::ByteSize(u64::try_from(code_bytes.len())?),
                bytesize::ByteSize(
                    account_view
                        .storage_usage
                        .checked_sub(u64::try_from(code_bytes.len())?)
                        .expect("Unexpected error")
                )
            ),
        ),
        (_code_hash, _global_account_id, _global_hash) => (
            "Contract",
            "Invalid account contract state. Please contact the developers."
                .red()
                .to_string(),
            String::new(),
            String::new(),
            format!("{}", bytesize::ByteSize(account_view.storage_usage)),
        ),
    };

    table.add_row(prettytable::row![
        Fy->contract_type,
        contract_status
    ]);

    table.add_row(prettytable::row![
        Fy->"SHA-256 checksum [hex]",
        checksum_hex
    ]);

    table.add_row(prettytable::row![
        Fy->"SHA-256 checksum [base58]",
        checksum_base58
    ]);

    table.add_row(prettytable::row![
        Fy->"Storage used",
        storage_used
    ]);

    let access_keys_summary = if access_keys.is_empty() {
        "Contract is locked (no access keys)".to_string()
    } else {
        let full_access_keys_count = access_keys
            .iter()
            .filter(|access_key| {
                matches!(
                    access_key.access_key.permission,
                    near_kit::AccessKeyPermissionView::FullAccess
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

    match get_contract_source_metadata(network_config, block_reference, account_id).await {
        Ok(contract_source_metadata) => {
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
        Err(err) => {
            table.add_row(prettytable::row![
                "",
                textwrap::fill(
                    &format!(
                        "{}: {}",
                        match &err {
                            FetchContractSourceMetadataError::ContractSourceMetadataNotSupported => "Info",
                            FetchContractSourceMetadataError::ContractSourceMetadataUnknownFormat(_) |
                            FetchContractSourceMetadataError::RpcError(_) => "Warning",
                        },
                        err
                    ),
                    80
                )
            ]);

            table.add_row(prettytable::row![
                Fy->"Contract version",
                "N/A"
            ]);
            table.add_row(prettytable::row![
                Fy->"Contract link",
                "N/A"
            ]);
            table.add_row(prettytable::row![
                Fy->"Supported standards",
                "N/A"
            ]);
        }
    }

    match super::get_contract_abi(network_config, block_reference, account_id).await {
        Ok(abi_root) => {
            table.add_row(prettytable::row![
                Fy->"NEAR ABI version",
                abi_root.schema_version
            ]);
            table.printstd();

            println!(
                "\n {} (hint: you can download full JSON Schema using `download-abi` command)",
                "Functions:".yellow()
            );
            for function in abi_root.body.functions {
                let mut table_func = prettytable::Table::new();
                table_func.set_format(*prettytable::format::consts::FORMAT_CLEAN);
                table_func.add_empty_row();

                table_func.add_row(prettytable::row![format!(
                    "{} ({}) {}\n{}",
                    format!(
                        "fn {}({}) -> {}",
                        function.name.green(),
                        "...".yellow(),
                        "...".blue()
                    ),
                    match function.kind {
                        near_abi::AbiFunctionKind::Call =>
                            "read-write function - transcation required",
                        near_abi::AbiFunctionKind::View => "read-only function",
                    },
                    function
                        .modifiers
                        .iter()
                        .fold(String::new(), |mut output, modifier| {
                            let _ = write!(
                                output,
                                "{} ",
                                match modifier {
                                    near_abi::AbiFunctionModifier::Init => "init".red(),
                                    near_abi::AbiFunctionModifier::Payable => "payable".red(),
                                    near_abi::AbiFunctionModifier::Private => "private".red(),
                                }
                            );
                            output
                        }),
                    function.doc.unwrap_or_default()
                )]);
                table_func.printstd();

                let mut table_args = prettytable::Table::new();
                table_args.set_format(*prettytable::format::consts::FORMAT_CLEAN);
                table_args.get_format().padding(1, 0);

                table_args.add_row(prettytable::row![
                    "...".yellow(),
                    Fy->"Arguments (JSON Schema):",
                ]);
                table_args.add_row(prettytable::row![
                    "   ",
                    if function.params.is_empty() {
                        "No arguments needed".to_string()
                    } else {
                        serde_json::to_string_pretty(&function.params).unwrap_or_default()
                    }
                ]);
                table_args.add_row(prettytable::row![
                    "...".blue(),
                    Fb->"Return Value (JSON Schema):",
                ]);
                table_args.add_row(prettytable::row![
                    "   ",
                    if let Some(result) = function.result {
                        serde_json::to_string_pretty(&result).unwrap_or_default()
                    } else {
                        "No return value".to_string()
                    }
                ]);
                table_args.printstd();
            }
        }
        Err(err) => {
            table.add_row(prettytable::row![
                Fy->"NEAR ABI version",
                textwrap::fill(
                    &format!(
                        "{}: {}",
                        match &err {
                            FetchAbiError::AbiNotSupported => "Info",
                            FetchAbiError::AbiUnknownFormat(_) | FetchAbiError::RpcError(_) => "Warning",
                        },
                        err
                    ),
                    80
                )
            ]);
            table.printstd();
            println!(
                "\n {} (NEAR ABI is not available, so only function names are extracted)\n",
                "Functions:".yellow()
            );

            let parser = wasmparser::Parser::new(0);
            for payload in parser.parse_all(&code_bytes) {
                if let wasmparser::Payload::ExportSection(export_section) =
                    payload.wrap_err_with(|| {
                        format!(
                            "Could not parse WebAssembly binary of the contract <{account_id}>."
                        )
                    })?
                {
                    for export in export_section {
                        let export = export
                            .wrap_err_with(|| format!("Could not parse WebAssembly export section of the contract <{account_id}>."))?;
                        if let wasmparser::ExternalKind::Func = export.kind {
                            println!(
                                " fn {}({}) -> {}\n",
                                export.name.green(),
                                "...".yellow(),
                                "...".blue()
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[tracing::instrument(name = "Getting information about", skip_all)]
async fn get_account_view(
    network_name: &str,
    network_config: &crate::config::NetworkConfig,
    block_reference: &BlockReference,
    account_id: &near_kit::AccountId,
) -> color_eyre::eyre::Result<near_kit::AccountView> {
    tracing::Span::current().pb_set_message(&format!("{account_id} ..."));
    tracing::info!(target: "near_teach_me", "Getting information about {account_id} ...");
    for _ in 0..5 {
        let result = query_view_account(
            network_config.client().rpc(),
            account_id,
            block_reference.clone(),
        )
        .await;

        match result {
            Ok(account_view) => {
                return Ok(account_view);
            }
            Err(ref err) if err.is_retryable() => {
                eprintln!(
                    "Transport error.\nPlease wait. The next try to send this query is happening right now ..."
                );
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(err) => {
                return Err(color_eyre::eyre::eyre!(
                    "Failed to fetch query ViewAccount for contract <{account_id}> on network <{network_name}>: {err}"
                ));
            }
        }
    }
    color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(format!(
        "Transport error. Failed to fetch query ViewAccount for contract <{account_id}> on network <{network_name}>"
    )))
}

#[tracing::instrument(name = "Getting a list of", skip_all)]
async fn get_access_keys(
    network_name: &str,
    network_config: &crate::config::NetworkConfig,
    block_reference: &BlockReference,
    account_id: &near_kit::AccountId,
) -> color_eyre::eyre::Result<Vec<near_kit::AccessKeyInfoView>> {
    tracing::Span::current().pb_set_message(&format!("{account_id} access keys ..."));
    tracing::info!(target: "near_teach_me", "Getting a list of {account_id} access keys ...");
    for _ in 0..5 {
        let result = query_view_access_key_list(
            network_config.client().rpc(),
            account_id,
            block_reference.clone(),
        )
        .await;

        match result {
            Ok(access_key_list) => {
                return Ok(access_key_list.keys);
            }
            Err(ref err) if err.is_retryable() => {
                eprintln!(
                    "Transport error.\nPlease wait. The next try to send this query is happening right now ..."
                );
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(err) => {
                return Err(color_eyre::eyre::eyre!(
                    "Failed to fetch ViewAccessKeyList for contract <{account_id}> on network <{network_name}>: {err}"
                ));
            }
        }
    }
    color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(format!(
        "Transport error. Failed to fetch query ViewAccessKeyList for contract <{account_id}> on network <{network_name}>"
    )))
}

#[derive(Error, Debug)]
pub enum FetchContractSourceMetadataError {
    #[error(
        "Contract Source Metadata (https://nomicon.io/Standards/SourceMetadata) is not supported by the contract, so there is no way to get detailed information."
    )]
    ContractSourceMetadataNotSupported,
    #[error(
        "'contract_source_metadata' function call failed due to RPC error, so there is no way to get Contract Source Metadata. See more details about the error:\n\n{0}"
    )]
    RpcError(String),
    #[error(
        "The contract source metadata format is unknown (https://nomicon.io/Standards/SourceMetadata), so there is no way to get detailed information. See more details about the error:\n\n{0}"
    )]
    ContractSourceMetadataUnknownFormat(Report),
}

#[tracing::instrument(name = "Getting contract source metadata for account", skip_all)]
pub async fn get_contract_source_metadata(
    network_config: &crate::config::NetworkConfig,
    block_reference: &BlockReference,
    account_id: &near_kit::AccountId,
) -> Result<
    near_verify_rs::types::contract_source_metadata::ContractSourceMetadata,
    FetchContractSourceMetadataError,
> {
    tracing::Span::current().pb_set_message(&format!("{account_id} ..."));
    tracing::info!(target: "near_teach_me", "Getting contract source metadata for account {account_id} ...");
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
            "I am making HTTP call to NEAR JSON RPC to call a read-only function `contract_source_metadata` on `{}` account, learn more https://docs.near.org/api/rpc/contracts#call-a-contract-function",
            account_id
    );

    let mut retries_left = (0..5).rev();
    loop {
        let result = query_view_function(
            network_config.client().rpc(),
            account_id,
            "contract_source_metadata",
            &[],
            block_reference.clone(),
        )
        .await;

        match result {
            Err(ref err) if err.is_retryable() => {
                if let Some(retries_left) = retries_left.next() {
                    sleep_after_error(format!(
                        "(Previous attempt failed with error: `{}`. Will retry {} more times)",
                        err.to_string().red(),
                        retries_left
                    ))
                    .await;
                } else {
                    return Err(FetchContractSourceMetadataError::RpcError(err.to_string()));
                }
            }
            Err(near_kit::RpcError::ContractExecution { message, .. })
                if message.contains("MethodNotFound") =>
            {
                return Err(FetchContractSourceMetadataError::ContractSourceMetadataNotSupported);
            }
            Err(near_kit::RpcError::FunctionCall { panic, .. })
                if panic.as_deref().unwrap_or("").contains("MethodNotFound") =>
            {
                return Err(FetchContractSourceMetadataError::ContractSourceMetadataNotSupported);
            }
            Err(err) => {
                return Err(FetchContractSourceMetadataError::RpcError(err.to_string()));
            }
            Ok(view_function_result) => {
                tracing::info!(
                    target: "near_teach_me",
                    parent: &tracing::Span::none(),
                    "JSON RPC Response:\n{}",
                    crate::common::indent_payload(&format!(
                        "{{\n  \"block_hash\": {}\n  \"block_height\": {}\n  \"logs\": {:?}\n  \"result\": {:?}\n}}",
                        view_function_result.block_hash,
                        view_function_result.block_height,
                        view_function_result.logs,
                        view_function_result.result
                    ))
                );
                tracing::info!(
                    target: "near_teach_me",
                    parent: &tracing::Span::none(),
                    "Decoding the \"result\" array of bytes as UTF-8 string (tip: you can use this Python snippet to do it: `\"\".join([chr(c) for c in result])`):\n{}",
                    crate::common::indent_payload(&format!("{}\n ",
                        &String::from_utf8(view_function_result.result.clone())
                            .unwrap_or_else(|_| "<decoding failed - the result is not a UTF-8 string>".to_owned())
                    ))
                );
                return view_function_result
                    .parse_result_from_json::<near_verify_rs::types::contract_source_metadata::ContractSourceMetadata>()
                    .wrap_err("Failed to parse contract source metadata")
                    .map_err(
                        FetchContractSourceMetadataError::ContractSourceMetadataUnknownFormat,
                    );
            }
        }
    }
}
