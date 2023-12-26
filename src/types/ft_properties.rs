use color_eyre::eyre::ContextCompat;
use color_eyre::eyre::WrapErr;

use crate::common::CallResultExt;
use crate::common::JsonRpcClientExt;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd)]
pub struct FtBalance {
    pub amount: u128,
    pub calculated_decimals: u64,
    pub ft_metadata: FtMetadata,
}

impl FtBalance {
    pub fn as_amount(&self) -> color_eyre::eyre::Result<u128> {
        if self.ft_metadata.decimals < self.calculated_decimals {
            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "Error: Invalid decimal places. Your FT amount exceeds <{}> decimal places.",
                self.ft_metadata.decimals
            ));
        }
        self.amount
            .checked_mul(10u128.pow((self.ft_metadata.decimals < self.calculated_decimals) as u32))
            .wrap_err("FT Balance: underflow or overflow happens")
    }
}

impl std::fmt::Display for FtBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let decimals = self.calculated_decimals;
        let one_ft: u128 = 10u128.pow(decimals as u32);
        if self.amount == 0 {
            write!(f, "0 {}", self.ft_metadata.symbol)
        } else if self.amount % one_ft == 0 {
            write!(f, "{} {}", self.amount / one_ft, self.ft_metadata.symbol)
        } else {
            write!(
                f,
                "{}.{} {}",
                self.amount / one_ft,
                format!(
                    "{:0>decimals$}",
                    (self.amount % one_ft),
                    decimals = decimals.try_into().unwrap()
                )
                .trim_end_matches('0'),
                self.ft_metadata.symbol
            )
        }
    }
}

impl std::str::FromStr for FtBalance {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num = s.trim().trim_end_matches(char::is_alphabetic).trim();
        let currency = s.trim().trim_start_matches(num).trim().to_uppercase();
        let res_split: Vec<&str> = num.split('.').collect();
        match res_split.len() {
            2 => {
                let num_int_part = res_split[0]
                    .parse::<u128>()
                    .map_err(|err| format!("FT Balance: {}", err))?;
                let len_fract = res_split[1].trim_end_matches('0').len() as u32;
                let num_fract_part = res_split[1]
                    .trim_end_matches('0')
                    .parse::<u128>()
                    .map_err(|err| format!("FT Balance: {}", err))?;
                let amount = num_int_part
                    .checked_mul(10u128.pow(len_fract))
                    .ok_or("FT Balance: underflow or overflow happens")?
                    .checked_add(num_fract_part)
                    .ok_or("FT Balance: underflow or overflow happens")?;
                Ok(FtBalance {
                    amount,
                    calculated_decimals: len_fract.into(),
                    ft_metadata: FtMetadata {
                        symbol: currency,
                        ..Default::default()
                    },
                })
            }
            1 => {
                if res_split[0].starts_with('0') && res_split[0] != "0" {
                    return Err("FT Balance: incorrect number entered".to_string());
                };
                let amount = res_split[0]
                    .parse::<u128>()
                    .map_err(|err| format!("FT Balance: {}", err))?;
                Ok(FtBalance {
                    amount,
                    calculated_decimals: 0,
                    ft_metadata: FtMetadata {
                        symbol: currency,
                        ..Default::default()
                    },
                })
            }
            _ => Err("FT Balance: incorrect number entered".to_string()),
        }
    }
}

impl interactive_clap::ToCli for FtBalance {
    type CliVariant = FtBalance;
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
        let ft_token = FtBalance::from_str("0 wNEAR").unwrap();
        assert_eq!(ft_token.to_string(), "0 WNEAR".to_string());
        assert_eq!(ft_token.calculated_decimals, 0)
    }
    #[test]
    fn ft_token_to_string_10_wnear() {
        let ft_token = FtBalance::from_str("10 wNEAR").unwrap();
        assert_eq!(ft_token.to_string(), "10 WNEAR".to_string());
        assert_eq!(ft_token.calculated_decimals, 0)
    }
    #[test]
    fn ft_token_to_string_0dot0200_wnear() {
        let ft_token = FtBalance::from_str("0.0200 wNEAR").unwrap();
        assert_eq!(ft_token.to_string(), "0.02 WNEAR".to_string());
        assert_eq!(ft_token.calculated_decimals, 2)
    }
    #[test]
    #[should_panic]
    fn ft_token_to_string_0dot1234567_usdc_invalid_decimals() {
        let ft_metadata = FtMetadata {
            symbol: "USDC".to_string(),
            decimals: 6,
        };
        let ft_token = FtBalance::from_str("0.1234567 USDC").unwrap();
        assert_eq!(ft_token.to_string(), "0.1234567 USDC".to_string());
        assert!(ft_token.calculated_decimals < ft_metadata.decimals)
    }
}
