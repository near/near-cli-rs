use color_eyre::eyre::Context;

use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

use crate::common::{CallResultExt, RpcQueryResponseExt, indent_payload};

pub type TokenId = String;

/// Asset identifiers supported by the MT-FT balance commands.
///
/// The program currently supports only tokens described by the NEP-141 and
/// NEP-245 protocols. Other asset identifier formats, such as `nep171:` and
/// `imt:`, are ignored when reading an owner's complete asset list.
#[derive(Debug, Clone, PartialEq)]
pub enum IntentsTokenId {
    Nep141(near_primitives::types::AccountId),
    Nep245(near_primitives::types::AccountId, TokenId),
}

impl IntentsTokenId {
    pub fn get_mt_ft_metadata(
        &self,
        network_config: &crate::config::NetworkConfig,
    ) -> color_eyre::eyre::Result<MtFtMetadata> {
        match &self {
            IntentsTokenId::Nep141(nep141_contract_id) => tokio::runtime::Runtime::new()
                .wrap_err("Failed to create a new tokio runtime")?
                .block_on(async {
                    nep141_mt_ft_metadata(
                        nep141_contract_id,
                        network_config,
                        near_primitives::types::Finality::Final.into(),
                    )
                    .await
                })
                .wrap_err_with(|| {
                    format!(
                        "Failed to get MT-FT metadata for token ID <{self}> on network <{}>",
                        network_config.network_name
                    )
                }),
            IntentsTokenId::Nep245(nep245_contract_id, token) => tokio::runtime::Runtime::new()
                .wrap_err("Failed to create a new tokio runtime")?
                .block_on(async {
                    nep245_mt_ft_metadata(
                        nep245_contract_id,
                        token.clone(),
                        network_config,
                        near_primitives::types::Finality::Final.into(),
                    )
                    .await
                })
                .wrap_err_with(|| {
                    format!(
                        "Failed to get MT-FT metadata for token ID <{self}> on network <{}>",
                        network_config.network_name
                    )
                }),
        }
    }
}

impl interactive_clap::ToCli for IntentsTokenId {
    type CliVariant = IntentsTokenId;
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

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, serde::Deserialize)]
pub struct MtFtMetadata {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
}

impl From<MtFtMetadata> for super::ft_properties::FtMetadata {
    fn from(mt_ft_metadata: MtFtMetadata) -> Self {
        Self {
            symbol: mt_ft_metadata.symbol,
            decimals: mt_ft_metadata.decimals,
        }
    }
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MtFtTransfer {
    pub receiver_id: near_primitives::types::AccountId,
    pub token_id: String,
    #[serde(deserialize_with = "parse_u128_string", serialize_with = "to_string")]
    pub amount: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval: Option<(near_primitives::types::AccountId, u64)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MtFtTransferCall {
    pub receiver_id: near_primitives::types::AccountId,
    pub token_id: String,
    #[serde(deserialize_with = "parse_u128_string", serialize_with = "to_string")]
    pub amount: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval: Option<(near_primitives::types::AccountId, u64)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    pub msg: String,
}

fn parse_u128_string<'de, D>(deserializer: D) -> color_eyre::eyre::Result<u128, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)?
        .parse::<u128>()
        .map_err(serde::de::Error::custom)
}

fn to_string<S, T: ToString>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = value.to_string();
    String::serialize(&s, serializer)
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
    fn test_invalid_token_id_unsupported_nep() {
        let result = IntentsTokenId::from_str("nep171:token.near:token");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("invalid token ID format"));
        assert!(err.contains("nep141"));
        assert!(err.contains("nep245"));
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
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let token_id: IntentsTokenId = serde_json::from_value(value["token_id"].clone()).unwrap();
        assert_eq!(
            token_id,
            IntentsTokenId::Nep141(
                near_primitives::types::AccountId::from_str("wrap.near").unwrap()
            )
        );
    }

    #[test]
    fn test_nep245_deserialize() {
        let json = r#"{"token_id":"nep245:v2.omni.near:1117_token"}"#;
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let token_id: IntentsTokenId = serde_json::from_value(value["token_id"].clone()).unwrap();
        assert_eq!(
            token_id,
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
        assert_eq!(metadata.name, "");
        assert_eq!(metadata.decimals, 0);
    }

    #[test]
    fn test_mt_ft_metadata_new() {
        let metadata = MtFtMetadata {
            symbol: "TON".to_string(),
            name: "Toncoin".to_string(),
            decimals: 9,
        };
        assert_eq!(metadata.symbol, "TON");
        assert_eq!(metadata.name, "Toncoin");
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
        assert_eq!(metadata.name, "Toncoin");
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
