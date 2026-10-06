use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

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
