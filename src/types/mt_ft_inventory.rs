use color_eyre::eyre::Context;
use tracing_indicatif::span_ext::IndicatifSpanExt;

use crate::common::{CallResultExt, JsonRpcClientExt, RpcQueryResponseExt, indent_payload};
use crate::types::mt_ft_properties::{IntentsTokenId, MtFtMetadata, TokenId};

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct FT {
    pub token_id: IntentsTokenId,
}

fn parse_supported_mt_fts(tokens: Vec<serde_json::Value>) -> Vec<FT> {
    tokens
        .into_iter()
        .filter_map(|token| match serde_json::from_value::<FT>(token) {
            Ok(token) => Some(token),
            Err(error) => {
                tracing::debug!(
                    "Skipping unsupported MT asset from 'mt_tokens_for_owner': {error}. The program currently only supports tokens described by the NEP-141 and NEP-245 protocols."
                );
                None
            }
        })
        .collect()
}

#[tracing::instrument(name = "Getting MT tokens for owner", skip_all, parent = None)]
pub fn get_mt_tokens_for_owner(
    network_config: &crate::config::NetworkConfig,
    mt_contract_account_id: &near_primitives::types::AccountId,
    owner_account_id: &near_primitives::types::AccountId,
    block_reference: near_primitives::types::BlockReference,
) -> color_eyre::eyre::Result<Vec<FT>> {
    tracing::Span::current().pb_set_message(&format!("account <{owner_account_id}> ..."));
    tracing::info!(target: "near_teach_me", "Getting MT tokens for owner account <{owner_account_id}> ...");

    let args = serde_json::to_vec(&serde_json::json!({
        "account_id": owner_account_id.clone().to_string(),
    }))?;
    network_config
        .json_rpc_client()
        .blocking_call_view_function(
            mt_contract_account_id,
            "mt_tokens_for_owner",
            args,
            block_reference,
        )
        .wrap_err_with(||{
            format!("Failed to fetch query for view method: 'mt_tokens_for_owner' (contract <{}> on network <{}>)",
                mt_contract_account_id,
                network_config.network_name
            )
        })?
        .parse_result_from_json::<Vec<serde_json::Value>>()
        .map(parse_supported_mt_fts)
        .wrap_err_with(||{
        format!("Failed to parse the result of the view method: 'mt_tokens_for_owner' (contract <{}> on network <{}>)",
            mt_contract_account_id,
            network_config.network_name
        )
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct MtFtInventory {
    pub token_id: IntentsTokenId,
    pub ft_token: crate::types::ft_properties::FungibleToken,
    pub price: Option<f64>,
}

pub async fn get_mt_ft_inventory(
    network_config: &crate::config::NetworkConfig,
    mt_contract: &near_primitives::types::AccountId,
    token_id: &IntentsTokenId,
    owner_account_id: &near_primitives::types::AccountId,
    block_reference: near_primitives::types::BlockReference,
) -> color_eyre::eyre::Result<MtFtInventory> {
    if let IntentsTokenId::Nep141(ft_contract_account_id) = token_id {
        let (mt_ft_metadata, amount_str) = tokio::join!(
            nep141_mt_ft_metadata(
                ft_contract_account_id,
                network_config,
                block_reference.clone(),
            ),
            get_mt_ft_balance(
                network_config,
                mt_contract,
                token_id,
                owner_account_id,
                block_reference.clone(),
            )
        );
        let mt_ft_metadata = mt_ft_metadata?;
        let amount_str = amount_str?;

        let fungible_token = crate::types::ft_properties::FungibleToken::from_params_ft(
            amount_str.parse::<u128>()?,
            mt_ft_metadata.decimals,
            mt_ft_metadata.symbol,
        );

        let mt_ft_inventory = MtFtInventory {
            token_id: token_id.clone(),
            ft_token: fungible_token,
            price: None, // Price retrieval logic can be added here if needed
        };

        Ok(mt_ft_inventory)
    } else if let IntentsTokenId::Nep245(ft_contract_account_id, token) = token_id {
        let (mt_ft_metadata, amount_str) = tokio::join!(
            nep245_mt_ft_metadata(
                ft_contract_account_id,
                token.clone(),
                network_config,
                block_reference.clone(),
            ),
            get_mt_ft_balance(
                network_config,
                mt_contract,
                token_id,
                owner_account_id,
                block_reference.clone(),
            )
        );
        let mt_ft_metadata = mt_ft_metadata?;
        let amount_str = amount_str?;

        let fungible_token = crate::types::ft_properties::FungibleToken::from_params_ft(
            amount_str.parse::<u128>()?,
            mt_ft_metadata.decimals,
            mt_ft_metadata.name,
        );

        let mt_ft_inventory = MtFtInventory {
            token_id: token_id.clone(),
            ft_token: fungible_token,
            price: None, // Price retrieval logic can be added here if needed
        };

        Ok(mt_ft_inventory)
    } else {
        Err(color_eyre::eyre::eyre!(
            "Unsupported token type: {token_id}. Only nep141 and nep245 token types are supported."
        ))
    }
}

#[tracing::instrument(name = "Getting MT-FT balance ...", skip_all, parent = None)]
pub async fn get_mt_ft_balance(
    network_config: &crate::config::NetworkConfig,
    mt_contract: &near_primitives::types::AccountId,
    token_id: &IntentsTokenId,
    owner_account_id: &near_primitives::types::AccountId,
    block_reference: near_primitives::types::BlockReference,
) -> color_eyre::eyre::Result<String> {
    tracing::info!(target: "near_teach_me", "Getting MT-FT balance ...");

    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "I am making HTTP call to NEAR JSON RPC to call the read-only function 'mt_balance_of' (contract <{token_id}>) for the account <{owner_account_id}>, learn more https://docs.near.org/api/rpc/contracts#call-a-contract-function",
    );

    let args = serde_json::to_vec(&serde_json::json!({
        "account_id": owner_account_id.clone().to_string(),
        "token_id": token_id.to_string(),
    }))?;

    let rpc_query_response = network_config
        .json_rpc_client()
        .call(
            near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference,
                request: near_primitives::views::QueryRequest::CallFunction {
                    account_id: mt_contract.clone(),
                    method_name: "mt_balance_of".to_string(),
                    args: near_primitives::types::FunctionArgs::from(args),
                }
            }
        )
        .await
        .wrap_err_with(||{
            format!("Failed to fetch query for view method: 'mt_balance_of' (contract <{}> on network <{}>)",
                mt_contract,
                network_config.network_name
            )
        })?;

    let call_result =rpc_query_response.call_result()
        .inspect(|call_result| {
            tracing::info!(
                target: "near_teach_me",
                parent: &tracing::Span::none(),
                "JSON RPC Response for 'mt_balance_of' (contract <{token_id}>) for the account <{owner_account_id}>:\n{}",
                indent_payload(&format!(
                    "{{\n  \"block_hash\": {}\n  \"block_height\": {}\n  \"logs\": {:?}\n  \"result\": {:?}\n}}",
                    rpc_query_response.block_hash,
                    rpc_query_response.block_height,
                    call_result.logs,
                    call_result.result
                ))
            );
            tracing::info!(
                target: "near_teach_me",
                parent: &tracing::Span::none(),
                "Decoding the \"result\" array of bytes as UTF-8 string (tip: you can use this Python snippet to do it: `\"\".join([chr(c) for c in result])`):\n{}",
                indent_payload(&format!("{}\n ", 
                    String::from_utf8(call_result.result.clone())
                        .unwrap_or_else(|_| "<decoding failed - the result is not a UTF-8 string>".to_owned())
                ))
            );
        })
        .inspect_err(|_| {
            tracing::info!(
                target: "near_teach_me",
                parent: &tracing::Span::none(),
                "JSON RPC Response for 'mt_balance_of' (contract <{token_id}>) for the account <{owner_account_id}>:\n{}",
                indent_payload("Internal error: Received unexpected query kind in response to a view-function query call")
            );
        })?;
    call_result.parse_result_from_json()
}

#[tracing::instrument(name = "Getting MT-FT metadata for nep141 contract ...", skip_all, parent = None)]
pub async fn nep141_mt_ft_metadata(
    ft_contract_account_id: &near_primitives::types::AccountId,
    network_config: &crate::config::NetworkConfig,
    block_reference: near_primitives::types::BlockReference,
) -> color_eyre::eyre::Result<MtFtMetadata> {
    tracing::info!(target: "near_teach_me", "Getting MT-FT metadata for nep141 contract ...");

    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "I am making HTTP call to NEAR JSON RPC to call the read-only function 'ft_metadata' for the contract <nep141:{ft_contract_account_id}>, learn more https://docs.near.org/api/rpc/contracts#call-a-contract-function",
    );

    let rpc_query_response = network_config
        .json_rpc_client()
        .call(
            near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference,
                request: near_primitives::views::QueryRequest::CallFunction {
                    account_id: ft_contract_account_id.clone(),
                    method_name: "ft_metadata".to_string(), 
                    args: near_primitives::types::FunctionArgs::from(vec![]),
                }
            }
        )
        .await
        .wrap_err_with(||{
            format!("Failed to fetch query for view method: 'ft_metadata' (contract <{}> on network <{}>)",
                ft_contract_account_id,
                network_config.network_name
            )
        })?;

    let call_result =rpc_query_response.call_result()
        .inspect(|call_result| {
            tracing::info!(
                target: "near_teach_me",
                parent: &tracing::Span::none(),
                "JSON RPC Response for 'ft_metadata' (contract <nep141:{ft_contract_account_id}>):\n{}",
                indent_payload(&format!(
                    "{{\n  \"block_hash\": {}\n  \"block_height\": {}\n  \"logs\": {:?}\n  \"result\": {:?}\n}}",
                    rpc_query_response.block_hash,
                    rpc_query_response.block_height,
                    call_result.logs,
                    call_result.result
                ))
            );
            tracing::info!(
                target: "near_teach_me",
                parent: &tracing::Span::none(),
                "Decoding the \"result\" array of bytes as UTF-8 string (tip: you can use this Python snippet to do it: `\"\".join([chr(c) for c in result])`):\n{}",
                indent_payload(&format!("{}\n ", 
                    String::from_utf8(call_result.result.clone())
                        .unwrap_or_else(|_| "<decoding failed - the result is not a UTF-8 string>".to_owned())
                ))
            );
        })
        .inspect_err(|_| {
            tracing::info!(
                target: "near_teach_me",
                parent: &tracing::Span::none(),
                "JSON RPC Response for 'ft_metadata' (contract <nep141:{ft_contract_account_id}>):\n{}",
                indent_payload("Internal error: Received unexpected query kind in response to a view-function query call")
            );
        })?;

    call_result.parse_result_from_json()
}

#[tracing::instrument(name = "Getting MT-FT metadata for nep245 contract ...", skip_all, parent = None)]
pub async fn nep245_mt_ft_metadata(
    nep245_contract_account_id: &near_primitives::types::AccountId,
    token_id: TokenId,
    network_config: &crate::config::NetworkConfig,
    block_reference: near_primitives::types::BlockReference,
) -> color_eyre::eyre::Result<MtFtMetadata> {
    tracing::info!(target: "near_teach_me", "Getting MT-FT metadata for nep245 contract ...");

    #[derive(serde::Deserialize)]
    struct MetadataResponse {
        base: MtFtMetadata,
    }

    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "I am making HTTP call to NEAR JSON RPC to call the read-only function 'mt_metadata_token_all' for the contract <nep245:{nep245_contract_account_id}:{token_id}>, learn more https://docs.near.org/api/rpc/contracts#call-a-contract-function",
    );

    let args = serde_json::to_vec(&serde_json::json!({
        "token_ids": [token_id],
    }))?;

    let rpc_query_response = network_config
        .json_rpc_client()
        .call(
            near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference,
                request: near_primitives::views::QueryRequest::CallFunction {
                    account_id: nep245_contract_account_id.clone(),
                    method_name: "mt_metadata_token_all".to_string(), 
                    args: near_primitives::types::FunctionArgs::from(args),
                }
            }
        )
        .await
        .wrap_err_with(||{
            format!("Failed to fetch query for view method: 'mt_metadata_token_all' (contract <{}> on network <{}>)",
                nep245_contract_account_id,
                network_config.network_name
            )
        })?;

    let call_result =rpc_query_response.call_result()
        .inspect(|call_result| {
            tracing::info!(
                target: "near_teach_me",
                parent: &tracing::Span::none(),
                "JSON RPC Response for 'mt_metadata_token_all' (contract <nep245:{}:{}>):\n{}",
                nep245_contract_account_id,
                token_id,
                indent_payload(&format!(
                    "{{\n  \"block_hash\": {}\n  \"block_height\": {}\n  \"logs\": {:?}\n  \"result\": {:?}\n}}",
                    rpc_query_response.block_hash,
                    rpc_query_response.block_height,
                    call_result.logs,
                    call_result.result
                ))
            );
            tracing::info!(
                target: "near_teach_me",
                parent: &tracing::Span::none(),
                "Decoding the \"result\" array of bytes as UTF-8 string (tip: you can use this Python snippet to do it: `\"\".join([chr(c) for c in result])`):\n{}",
                indent_payload(&format!("{}\n ", 
                    String::from_utf8(call_result.result.clone())
                        .unwrap_or_else(|_| "<decoding failed - the result is not a UTF-8 string>".to_owned())
                ))
            );
        })
        .inspect_err(|_| {
            tracing::info!(
                target: "near_teach_me",
                parent: &tracing::Span::none(),
                "JSON RPC Response for 'mt_metadata_token_all' (contract <nep245:{}:{}>):\n{}",
                nep245_contract_account_id,
                token_id,
                indent_payload("Internal error: Received unexpected query kind in response to a view-function query call")
            );
        })?;

    // Parse as array and get first element
    let metadata_array: Vec<MetadataResponse> = call_result.parse_result_from_json()?;
    let mt_ft_metadata = metadata_array
        .into_iter()
        .next()
        .ok_or_else(|| color_eyre::eyre::eyre!("Empty metadata array returned"))?
        .base;

    Ok(mt_ft_metadata)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_parse_supported_tokens_skips_unsupported_assets() {
        let tokens = vec![
            serde_json::json!({ "token_id": "nep171:token.near:token" }),
            serde_json::json!({ "token_id": "nep245:v2.omni.near:1117_token" }),
            serde_json::json!({ "token_id": "imt:token.near:token" }),
        ];

        let parsed_tokens = parse_supported_mt_fts(tokens);

        assert_eq!(parsed_tokens.len(), 1);
        assert_eq!(
            parsed_tokens[0].token_id,
            IntentsTokenId::Nep245(
                near_primitives::types::AccountId::from_str("v2.omni.near").unwrap(),
                "1117_token".to_string()
            )
        );
    }
}
