use color_eyre::eyre::Context;
use tracing_indicatif::span_ext::IndicatifSpanExt;

use crate::common::{CallResultExt, JsonRpcClientExt, RpcQueryResponseExt, indent_payload};

type TokenId = String;

#[derive(Debug, Clone, PartialEq)]
pub enum IntentsTokenId {
    Nep141(near_primitives::types::AccountId),
    Nep245(near_primitives::types::AccountId, TokenId),
}

impl<'de> serde::Deserialize<'de> for IntentsTokenId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let token_id = String::deserialize(deserializer)?;
        token_id.parse().map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for IntentsTokenId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nep141(account_id) => write!(f, "nep141:{account_id}"),
            Self::Nep245(account_id, token_id) => write!(f, "nep245:{account_id}:{token_id}"),
        }
    }
}

impl std::str::FromStr for IntentsTokenId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.get(.."nep141:".len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("nep141:"))
        {
            let account_id = &s["nep141:".len()..];
            account_id
                .parse::<near_primitives::types::AccountId>()
                .map(Self::Nep141)
                .map_err(|e| e.to_string())
        } else if s
            .get(.."nep245:".len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("nep245:"))
        {
            let rest = &s["nep245:".len()..];
            let (account_id, token_id) = rest.split_once(':').unwrap_or((rest, ""));

            if token_id.is_empty() {
                return Err("nep245: token_id cannot be empty".to_string());
            }

            account_id
                .parse::<near_primitives::types::AccountId>()
                .map(|a| Self::Nep245(a, token_id.to_string()))
                .map_err(|e| e.to_string())
        } else {
            Err(format!(
                "invalid token ID format `{s}`: expected `nep141:<account_id>` or `nep245:<account_id>:<token_id>`",
                s = s
            ))
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct FT {
    pub token_id: IntentsTokenId,
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
        .parse_result_from_json()
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
                ft_contract_account_id.clone(),
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
            mt_ft_metadata.symbol.clone(),
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
                ft_contract_account_id.clone(),
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
            mt_ft_metadata
                .name
                .clone()
                .unwrap_or_else(|| mt_ft_metadata.symbol.clone()),
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
async fn get_mt_ft_balance(
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

#[derive(Debug, Clone, Default, PartialEq, serde::Deserialize)]
pub struct MtFtMetadata {
    symbol: String,
    name: Option<String>,
    decimals: u8,
}

#[tracing::instrument(name = "Getting MT-FT metadata for nep141 contract ...", skip_all, parent = None)]
async fn nep141_mt_ft_metadata(
    ft_contract_account_id: near_primitives::types::AccountId,
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
async fn nep245_mt_ft_metadata(
    nep245_contract_account_id: near_primitives::types::AccountId,
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
    fn test_nep141_token_id_parsing() {
        let token_id = IntentsTokenId::from_str("nep141:wrap.near").unwrap();
        assert_eq!(
            token_id,
            IntentsTokenId::Nep141(
                near_primitives::types::AccountId::from_str("wrap.near").unwrap()
            )
        );
    }

    #[test]
    fn test_nep141_token_id_case_insensitive() {
        let token_id1 = IntentsTokenId::from_str("NEP141:wrap.near").unwrap();
        let token_id2 = IntentsTokenId::from_str("nep141:wrap.near").unwrap();
        assert_eq!(token_id1, token_id2);
    }

    #[test]
    fn test_nep245_token_id_parsing() {
        let token_id = IntentsTokenId::from_str("nep245:v2.omni.near:1117_token_id").unwrap();
        assert_eq!(
            token_id,
            IntentsTokenId::Nep245(
                near_primitives::types::AccountId::from_str("v2.omni.near").unwrap(),
                "1117_token_id".to_string()
            )
        );
    }

    #[test]
    fn test_nep245_token_id_with_multiple_colons() {
        let token_id = IntentsTokenId::from_str(
            "nep245:v2.omni.near:1117_3tsdfyziyc7EJbP2aULWSKU4toBaAcN4FdTgfm5W1mC4ouR",
        )
        .unwrap();
        assert_eq!(
            token_id,
            IntentsTokenId::Nep245(
                near_primitives::types::AccountId::from_str("v2.omni.near").unwrap(),
                "1117_3tsdfyziyc7EJbP2aULWSKU4toBaAcN4FdTgfm5W1mC4ouR".to_string()
            )
        );
    }

    #[test]
    fn test_nep245_token_id_case_insensitive() {
        let token_id1 = IntentsTokenId::from_str("NEP245:v2.omni.near:token").unwrap();
        let token_id2 = IntentsTokenId::from_str("nep245:v2.omni.near:token").unwrap();
        assert_eq!(token_id1, token_id2);
    }

    #[test]
    fn test_invalid_token_id_no_prefix() {
        let result = IntentsTokenId::from_str("wrap.near");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("invalid token ID format"));
    }

    #[test]
    fn test_invalid_token_id_incomplete_nep141() {
        let result = IntentsTokenId::from_str("nep141:");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_token_id_incomplete_nep245() {
        let result = IntentsTokenId::from_str("nep245:v2.omni.near:");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_account_id_in_nep141() {
        let result = IntentsTokenId::from_str("nep141:invalid..near");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_account_id_in_nep245() {
        let result = IntentsTokenId::from_str("nep245:invalid..near:token");
        assert!(result.is_err());
    }

    #[test]
    fn test_nep141_display() {
        let token_id = IntentsTokenId::Nep141(
            near_primitives::types::AccountId::from_str("wrap.near").unwrap(),
        );
        assert_eq!(token_id.to_string(), "nep141:wrap.near");
    }

    #[test]
    fn test_nep245_display() {
        let token_id = IntentsTokenId::Nep245(
            near_primitives::types::AccountId::from_str("v2.omni.near").unwrap(),
            "1117_token_id".to_string(),
        );
        assert_eq!(token_id.to_string(), "nep245:v2.omni.near:1117_token_id");
    }

    #[test]
    fn test_nep141_deserialize() {
        let json = r#"{"token_id":"nep141:wrap.near"}"#;
        let ft: FT = serde_json::from_str(json).unwrap();
        assert_eq!(
            ft.token_id,
            IntentsTokenId::Nep141(
                near_primitives::types::AccountId::from_str("wrap.near").unwrap()
            )
        );
    }

    #[test]
    fn test_nep245_deserialize() {
        let json = r#"{"token_id":"nep245:v2.omni.near:1117_token"}"#;
        let ft: FT = serde_json::from_str(json).unwrap();
        assert_eq!(
            ft.token_id,
            IntentsTokenId::Nep245(
                near_primitives::types::AccountId::from_str("v2.omni.near").unwrap(),
                "1117_token".to_string()
            )
        );
    }

    #[test]
    fn test_mt_ft_metadata_default() {
        let metadata = MtFtMetadata::default();
        assert_eq!(metadata.symbol, "");
        assert_eq!(metadata.name, None);
        assert_eq!(metadata.decimals, 0);
    }

    #[test]
    fn test_mt_ft_metadata_new() {
        let metadata = MtFtMetadata {
            symbol: "TON".to_string(),
            name: Some("Toncoin".to_string()),
            decimals: 9,
        };
        assert_eq!(metadata.symbol, "TON");
        assert_eq!(metadata.name, Some("Toncoin".to_string()));
        assert_eq!(metadata.decimals, 9);
    }

    #[test]
    fn test_mt_ft_metadata_deserialize() {
        let json = r#"{
            "symbol": "TON",
            "name": "Toncoin",
            "decimals": 9
        }"#;
        let metadata: MtFtMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.symbol, "TON");
        assert_eq!(metadata.name, Some("Toncoin".to_string()));
        assert_eq!(metadata.decimals, 9);
    }

    #[test]
    fn test_mt_ft_metadata_deserialize_without_name() {
        let json = r#"{
            "symbol": "TON",
            "decimals": 9
        }"#;
        let metadata: MtFtMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.symbol, "TON");
        assert_eq!(metadata.name, None);
        assert_eq!(metadata.decimals, 9);
    }

    #[test]
    fn test_round_trip_nep141() {
        let original = "nep141:wrap.near";
        let token_id = IntentsTokenId::from_str(original).unwrap();
        assert_eq!(token_id.to_string(), original);
    }

    #[test]
    fn test_round_trip_nep245() {
        let original = "nep245:v2.omni.near:1117_token";
        let token_id = IntentsTokenId::from_str(original).unwrap();
        assert_eq!(token_id.to_string(), original);
    }
}
