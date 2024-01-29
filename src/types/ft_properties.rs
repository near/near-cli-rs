use color_eyre::eyre::{Context, ContextCompat};

use crate::common::CallResultExt;
use crate::common::JsonRpcClientExt;

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
        if ft_metadata.decimals < u64::from(self.decimals) {
            color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "Invalid decimal places. Your FT amount exceeds {} decimal places.",
                ft_metadata.decimals
            ))
        } else if ft_metadata.symbol.to_uppercase() != self.symbol.to_uppercase() {
            color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!("Invalid currency symbol"))
        } else if ft_metadata.symbol == self.symbol
            && ft_metadata.decimals == u64::from(self.decimals)
        {
            Ok(self.clone())
        } else {
            let mut fungible_token = self.clone();
            let actual_decimals = u32::try_from(ft_metadata.decimals)
                .wrap_err("Error converting u64 to u32")?
                .checked_sub(fungible_token.decimals().into())
                .wrap_err("FT Balance: underflow or overflow happens")?;
            fungible_token.amount = fungible_token
                .amount
                .checked_mul(
                    10u128
                        .checked_pow(actual_decimals)
                        .wrap_err("FT Balance: overflow happens")?,
                )
                .wrap_err("FungibleToken: overflow happens")?;
            fungible_token.decimals = ft_metadata
                .decimals
                .try_into()
                .wrap_err("Error converting u64 to u8")?;
            fungible_token.symbol = ft_metadata.symbol.clone();
            Ok(fungible_token)
        }
    }

    pub fn amount(&self) -> u128 {
        self.amount
    }

    pub fn decimals(&self) -> u8 {
        self.decimals
    }

    pub fn symbol(&self) -> String {
        self.symbol.to_owned()
    }
}

impl std::fmt::Display for FungibleToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let one_ft: u128 = 10u128
            .checked_pow(self.decimals.into())
            .wrap_err("FT Balance: overflow happens")
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
                    .map_err(|err| format!("FungibleToken: {}", err))?;
                let len_fract: u8 = res_split[1]
                    .trim_end_matches('0')
                    .len()
                    .try_into()
                    .map_err(|_| "Error converting usize to u8")?;
                let num_fract_part = res_split[1]
                    .trim_end_matches('0')
                    .parse::<u128>()
                    .map_err(|err| format!("FungibleToken: {}", err))?;
                let amount = num_int_part
                    .checked_mul(
                        10u128
                            .checked_pow(len_fract.into())
                            .ok_or("FT Balance: overflow happens")?,
                    )
                    .ok_or("FungibleToken: overflow happens")?
                    .checked_add(num_fract_part)
                    .ok_or("FungibleToken: overflow happens")?;
                Ok(FungibleToken {
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
                    .map_err(|err| format!("FungibleToken: {}", err))?;
                Ok(FungibleToken {
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
    pub decimals: u64,
}

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
}
