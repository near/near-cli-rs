use color_eyre::eyre::{Context, ContextCompat};
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

use crate::common::CallResultExt;
use crate::common::JsonRpcClientExt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum FungibleTokenTransferAmount {
    /// Transfer of the specified amount of fungible tokens (wNearAmount (10 wNEAR))
    ExactAmount(FungibleToken),
    /// Transfer the entire amount of fungible tokens from your account ID
    MaxAmount,
}

impl interactive_clap::ToCli for FungibleTokenTransferAmount {
    type CliVariant = FungibleTokenTransferAmount;
}

impl FungibleTokenTransferAmount {
    pub fn normalize(&self, ft_metadata: &FtMetadata) -> color_eyre::eyre::Result<Self> {
        if let Self::ExactAmount(ft) = self {
            Ok(Self::ExactAmount(ft.normalize(ft_metadata)?))
        } else {
            Ok(Self::MaxAmount)
        }
    }
}

impl std::fmt::Display for FungibleTokenTransferAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExactAmount(ft) => ft.fmt(f),
            Self::MaxAmount => write!(f, "all"),
        }
    }
}

impl std::str::FromStr for FungibleTokenTransferAmount {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.to_lowercase() == "all" {
            Ok(Self::MaxAmount)
        } else {
            Ok(Self::ExactAmount(FungibleToken::from_str(s)?))
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd)]
pub struct FungibleToken {
    amount: u128,
    decimals: u8,
    symbol: String,
}

impl FungibleToken {
    pub fn from_params_ft(amount: u128, decimals: u8, symbol: String) -> Self {
        Self {
            amount,
            decimals,
            symbol,
        }
    }

    pub fn normalize(&self, ft_metadata: &FtMetadata) -> color_eyre::eyre::Result<Self> {
        if ft_metadata.symbol.to_uppercase() != self.symbol.to_uppercase() {
            color_eyre::eyre::bail!("Invalid currency symbol")
        } else if let Some(decimals_diff) = ft_metadata.decimals.checked_sub(self.decimals) {
            let amount = if decimals_diff == 0 {
                self.amount
            } else {
                self.amount
                    .checked_mul(
                        10u128
                            .checked_pow(decimals_diff.into())
                            .wrap_err("Overflow in decimal normalization")?,
                    )
                    .wrap_err("Overflow in decimal normalization")?
            };
            Ok(Self {
                symbol: ft_metadata.symbol.clone(),
                decimals: ft_metadata.decimals,
                amount,
            })
        } else {
            color_eyre::eyre::bail!(
                "Invalid decimal places. Your FT amount exceeds {} decimal places.",
                ft_metadata.decimals
            )
        }
    }

    pub fn amount(&self) -> u128 {
        self.amount
    }

    pub fn decimals(&self) -> u8 {
        self.decimals
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }
}

impl std::fmt::Display for FungibleToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let one_ft: u128 = 10u128
            .checked_pow(self.decimals.into())
            .wrap_err("Overflow in FungibleToken normalization")
            .unwrap();
        if self.amount == 0 {
            write!(f, "0 {}", self.symbol)
        } else if self.amount % one_ft == 0 {
            write!(f, "{} {}", self.amount / one_ft, self.symbol)
        } else {
            write!(
                f,
                "{}.{} {}",
                self.amount / one_ft,
                format!(
                    "{:0>decimals$}",
                    (self.amount % one_ft),
                    decimals = self.decimals.into()
                )
                .trim_end_matches('0'),
                self.symbol
            )
        }
    }
}

impl std::str::FromStr for FungibleToken {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num = s.trim().trim_end_matches(char::is_alphabetic).trim();
        let currency = s.trim().trim_start_matches(num).trim().to_string();
        let res_split: Vec<&str> = num.split('.').collect();
        match res_split.len() {
            2 => {
                let num_int_part = res_split[0]
                    .parse::<u128>()
                    .map_err(|err| format!("FungibleToken: {err}"))?;
                let len_fract: u8 = res_split[1]
                    .trim_end_matches('0')
                    .len()
                    .try_into()
                    .map_err(|_| "Error converting len_fract to u8")?;
                let num_fract_part = res_split[1]
                    .trim_end_matches('0')
                    .parse::<u128>()
                    .map_err(|err| format!("FungibleToken: {err}"))?;
                let amount = num_int_part
                    .checked_mul(
                        10u128
                            .checked_pow(len_fract.into())
                            .ok_or("FT Balance: overflow happens")?,
                    )
                    .ok_or("FungibleToken: overflow happens")?
                    .checked_add(num_fract_part)
                    .ok_or("FungibleToken: overflow happens")?;
                Ok(Self {
                    amount,
                    decimals: len_fract,
                    symbol: currency,
                })
            }
            1 => {
                if res_split[0].starts_with('0') && res_split[0] != "0" {
                    return Err("FungibleToken: incorrect number entered".to_string());
                };
                let amount = res_split[0]
                    .parse::<u128>()
                    .map_err(|err| format!("FungibleToken: {err}"))?;
                Ok(Self {
                    amount,
                    decimals: 0,
                    symbol: currency,
                })
            }
            _ => Err("FungibleToken: incorrect number entered".to_string()),
        }
    }
}

impl interactive_clap::ToCli for FungibleToken {
    type CliVariant = FungibleToken;
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, serde::Deserialize)]
pub struct FtMetadata {
    pub symbol: String,
    pub decimals: u8,
}

#[tracing::instrument(name = "Getting FT metadata ...", skip_all, parent = None)]
pub fn params_ft_metadata(
    ft_contract_account_id: near_primitives::types::AccountId,
    network_config: &crate::config::NetworkConfig,
    block_reference: near_primitives::types::BlockReference,
) -> color_eyre::eyre::Result<FtMetadata> {
    let ft_metadata: FtMetadata = network_config
        .json_rpc_client()
        .blocking_call_view_function(
            &ft_contract_account_id,
            "ft_metadata",
            vec![],
            block_reference,
        )
        .wrap_err_with(||{
            format!("Failed to fetch query for view method: 'ft_metadata' (contract <{}> on network <{}>)",
                ft_contract_account_id,
                network_config.network_name
            )
        })?
        .parse_result_from_json()?;
    Ok(ft_metadata)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FtTransfer {
    pub receiver_id: near_primitives::types::AccountId,
    #[serde(deserialize_with = "parse_u128_string", serialize_with = "to_string")]
    pub amount: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
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
    fn ft_token_to_string_0_wnear() {
        let ft_token = FungibleToken::from_str("0 wNEAR").unwrap();
        assert_eq!(ft_token.to_string(), "0 wNEAR".to_string());
        assert_eq!(ft_token.symbol, "wNEAR".to_string());
        assert_eq!(ft_token.decimals, 0)
    }
    #[test]
    fn ft_token_to_string_10_wnear() {
        let ft_token = FungibleToken::from_str("10 wNEAR").unwrap();
        assert_eq!(ft_token.to_string(), "10 wNEAR".to_string());
        assert_eq!(ft_token.symbol, "wNEAR".to_string());
        assert_eq!(ft_token.decimals, 0)
    }
    #[test]
    fn ft_token_to_string_0dot0200_wnear() {
        let ft_token = FungibleToken::from_str("0.0200 wNEAR").unwrap();
        assert_eq!(ft_token.to_string(), "0.02 wNEAR".to_string());
        assert_eq!(ft_token.symbol, "wNEAR".to_string());
        assert_eq!(ft_token.decimals, 2)
    }
    #[test]
    fn ft_token_to_string_0dot123456_usdc() {
        let ft_token = FungibleToken::from_str("0.123456 USDC").unwrap();
        assert_eq!(ft_token.to_string(), "0.123456 USDC".to_string());
        assert_eq!(ft_token.symbol, "USDC".to_string());
    }
    #[test]
    fn ft_transfer_amount_to_string_0dot123456_usdc() {
        let ft_transfer_amount = FungibleTokenTransferAmount::from_str("0.123456 USDC").unwrap();
        assert_eq!(ft_transfer_amount.to_string(), "0.123456 USDC".to_string());
        assert_eq!(
            ft_transfer_amount,
            FungibleTokenTransferAmount::ExactAmount(FungibleToken::from_params_ft(
                123456,
                6,
                "USDC".to_string()
            ))
        );
    }
    #[test]
    fn ft_transfer_amount_to_string_all() {
        let ft_transfer_amount = FungibleTokenTransferAmount::from_str("all").unwrap();
        assert_eq!(ft_transfer_amount.to_string(), "all".to_string());
        assert_eq!(ft_transfer_amount, FungibleTokenTransferAmount::MaxAmount);
    }
    #[test]
    fn ft_transfer_with_memo_to_vec_u8() {
        let ft_transfer = serde_json::to_vec(&crate::types::ft_properties::FtTransfer {
            receiver_id: "fro_volod.testnet".parse().unwrap(),
            amount: FungibleToken::from_str("0.123456 USDC").unwrap().amount(),
            memo: Some("Memo".to_string()),
        })
        .unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&ft_transfer)
                .unwrap()
                .to_string(),
            "{\"amount\":\"123456\",\"memo\":\"Memo\",\"receiver_id\":\"fro_volod.testnet\"}"
        );
    }
    #[test]
    fn ft_transfer_without_memo_to_vec_u8() {
        let ft_transfer = serde_json::to_vec(&crate::types::ft_properties::FtTransfer {
            receiver_id: "fro_volod.testnet".parse().unwrap(),
            amount: FungibleToken::from_str("0.123456 USDC").unwrap().amount(),
            memo: None,
        })
        .unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&ft_transfer)
                .unwrap()
                .to_string(),
            "{\"amount\":\"123456\",\"receiver_id\":\"fro_volod.testnet\"}"
        );
    }
}
