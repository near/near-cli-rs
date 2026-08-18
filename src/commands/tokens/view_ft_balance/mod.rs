use color_eyre::eyre::Context;
use serde_json::json;

use crate::common::CallResultExt;
use crate::common::{RpcResultExt, block_on};

use super::send_ft::input_ft_contract_account_id;
use crate::types::ft_inventory::{FTContract, FTInventory, get_account_ft_inventory};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::TokensCommandsContext)]
#[interactive_clap(output_context = ViewFtBalanceContext)]
pub struct ViewFtBalance {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the ft-contract account ID?
    ft_contract: FTContract,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct ViewFtBalanceContext(crate::network_view_at_block::ArgsForViewContext);

impl ViewFtBalanceContext {
    pub fn from_previous_context(
        previous_context: super::TokensCommandsContext,
        scope: &<ViewFtBalance as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let owner_account_id = previous_context.owner_account_id.clone();
            let ft_contract = scope.ft_contract.clone();
            let credentials_home_dir = previous_context.global_context.config.credentials_home_dir.clone();

            move |network_config, block_reference| {
                if let FTContract::SingleContract(ft_contract_account_id) = &ft_contract {
                    let ft_contract_account_id: near_kit::AccountId = ft_contract_account_id.clone().into();
                    let ft_metadata = crate::types::ft_properties::params_ft_metadata(
                        ft_contract_account_id.clone(),
                        network_config,
                        *block_reference,
                    )?;

                    let ft_contract = crate::types::ft_properties::FtContract {
                        ft_metadata: ft_metadata.clone(),
                        ft_contract_account_id: ft_contract_account_id.clone(),
                    };

                    crate::common::update_used_ft_contract_account_list(
                        &credentials_home_dir,
                        &ft_contract,
                    );

                    let args = serde_json::to_vec(&json!({
                        "account_id": owner_account_id.clone().to_string(),
                        }))?;
                    let call_result = get_ft_balance(network_config, &ft_contract_account_id, args, *block_reference)?;
                    call_result.print_logs();
                    let amount: String = call_result.parse_result_from_json()?;
                    let fungible_token = crate::types::ft_properties::FungibleToken::from_params_ft(
                        amount.parse::<u128>()?,
                        ft_metadata.decimals,
                        ft_metadata.symbol
                    );

                    println!("<{owner_account_id}> account has {fungible_token}  (FT-contract: {ft_contract_account_id})");
                } else {
                    print_fts_inventory(network_config, &owner_account_id, previous_context.global_context.verbosity)?;
                }
                Ok(())
            }
            });
        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.global_context.config,
            on_after_getting_block_reference_callback,
            interacting_with_account_ids: vec![previous_context.owner_account_id],
        }))
    }
}

impl From<ViewFtBalanceContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: ViewFtBalanceContext) -> Self {
        item.0
    }
}

impl ViewFtBalance {
    pub fn input_ft_contract(
        context: &super::TokensCommandsContext,
    ) -> color_eyre::eyre::Result<Option<FTContract>> {
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(
                to_string = "Yes, I want to view the balance for a specific fungible token (FT)."
            )]
            Yes,
            #[strum(
                to_string = "No,  I want to view balances for all fungible tokens for this account."
            )]
            No,
        }

        let select_choose_input = inquire::Select::new(
            "Do you want to view the balance of a specific fungible token (FT) for this account?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let Some(ft_contract_account_id) =
                input_ft_contract_account_id(&context.global_context.config.credentials_home_dir)?
            else {
                return Ok(None);
            };
            Ok(Some(FTContract::SingleContract(ft_contract_account_id)))
        } else {
            Ok(Some(FTContract::AllContracts))
        }
    }
}

#[tracing::instrument(name = "Getting FT balance ...", skip_all, parent = None)]
pub fn get_ft_balance(
    network_config: &crate::config::NetworkConfig,
    ft_contract_account_id: &near_kit::AccountId,
    args: Vec<u8>,
    block_reference: near_kit::BlockReference,
) -> color_eyre::eyre::Result<near_kit::ViewFunctionResult> {
    tracing::info!(target: "near_teach_me", "Getting FT balance ...");
    let result = block_on(
            network_config.client().rpc().view_function(
                ft_contract_account_id,
                "ft_balance_of",
                &args,
                block_reference,
            ),
        )
        .into_eyre()
        .wrap_err_with(||{
            format!("Failed to fetch query for view method: 'ft_balance_of' (contract <{}> on network <{}>)",
                ft_contract_account_id,
                network_config.network_name
            )
        })?;
    Ok(result)
}

#[derive(Debug, Clone, PartialEq)]
struct FTCalculatedValue {
    ft: FTInventory,
    tokens_scaled: String, // Exact formatted string for values of any scale
    usd_value: f64,        // Used strictly for sorting and filtering
}

fn parse_raw_amount_to_string(amount_str: &str, decimals: u8) -> Option<String> {
    let decimals = decimals as usize;

    if amount_str.is_empty() || !amount_str.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    if decimals == 0 {
        return Some(amount_str.to_string());
    }

    let min_len = decimals + 1;
    let padded = if amount_str.len() < min_len {
        format!("{:0>width$}", amount_str, width = min_len)
    } else {
        amount_str.to_string()
    };

    let split_pos = padded.len() - decimals;
    let (integer_str, fraction_str) = padded.split_at(split_pos);

    let trimmed_fraction = fraction_str.trim_end_matches('0');

    if trimmed_fraction.is_empty() {
        Some(integer_str.to_string())
    } else {
        Some(format!("{integer_str}.{trimmed_fraction}"))
    }
}

fn calculate_usd_value_f64(amount_str: &str, decimals: u8, price: f64) -> Option<f64> {
    if amount_str.is_empty() || !amount_str.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let raw_amount: f64 = amount_str.parse().ok()?;
    let divisor = 10.0_f64.powi(decimals as i32);
    let usd = (raw_amount / divisor) * price;

    if usd.is_finite() && usd >= 0.0 {
        Some(usd)
    } else {
        None
    }
}

fn calculate_ft_usd_value(ft: &FTInventory) -> Option<FTCalculatedValue> {
    let price = ft.ft_meta.price?;

    let usd_value = calculate_usd_value_f64(&ft.amount, ft.ft_meta.decimals, price)?;
    let tokens_scaled = parse_raw_amount_to_string(&ft.amount, ft.ft_meta.decimals)?;

    Some(FTCalculatedValue {
        ft: ft.clone(),
        tokens_scaled,
        usd_value,
    })
}

fn print_fts_inventory(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_kit::AccountId,
    verbosity: crate::Verbosity,
) -> crate::CliResult {
    let inventory = get_account_ft_inventory(network_config, account_id)?;
    let min_usd = 0.1; // $0.10

    let mut fts = inventory
        .fts()
        .into_iter()
        .filter_map(|ft| calculate_ft_usd_value(&ft).filter(|item| item.usd_value >= min_usd))
        .collect::<Vec<_>>();

    if fts.is_empty() {
        if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe = verbosity {
            eprintln!(
                "The account <{account_id}> has no fungible tokens worth at least $0.10 on network <{}>.",
                network_config.network_name
            );
        }
        return Ok(());
    }

    fts.sort_by(|a, b| {
        b.usd_value
            .partial_cmp(&a.usd_value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let output = fts
        .iter()
        .map(|item| {
            format!(
                "\t{} {} (FT-contract: {})\n",
                item.tokens_scaled, item.ft.ft_meta.symbol, item.ft.ft_contract_account_id
            )
        })
        .collect::<String>();

    if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe = verbosity {
        eprintln!(
            "The account <{account_id}> has fungible tokens worth at least $0.10 (printed to stdout):"
        );
    }

    print!("{output}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // Helper function for quickly creating test objects
    fn create_test_ft(amount: &str, decimals: u8, price: Option<f64>) -> FTInventory {
        FTInventory {
            amount: amount.to_string(),
            ft_contract_account_id: near_kit::AccountId::from_str("test.near").unwrap(),
            ft_meta: crate::types::ft_inventory::FTMeta {
                decimals,
                name: "Test Token".to_string(),
                price,
                symbol: "TT".to_string(),
            },
        }
    }

    #[test]
    fn test_normal_calculation() {
        // Case: 100.5 tokens (2 decimals), price $2.50
        // 10050 base units / 10^2 = 100.5 tokens. 100.5 * 2.50 = 251.25
        let ft = create_test_ft("10050", 2, Some(2.50));
        let result = calculate_ft_usd_value(&ft).unwrap();

        assert_eq!(result.tokens_scaled, "100.5");
        assert!((result.usd_value - 251.25).abs() < 1e-6);
    }

    #[test]
    fn test_amount_exceeding_decimal_max_with_tiny_price() {
        // Case: amount string is vastly larger than u128::MAX and Decimal::MAX, but price is tiny (10^-10).
        // String-based tokens_scaled retains exact precision for CLI output, while f64 calculates estimated USD.
        let crazy_amount = "34028236692093846346337460743176821145600000";
        let tiny_price = Some(0.0000000001); // 10^-10

        let ft = create_test_ft(crazy_amount, 2, tiny_price);
        let result = calculate_ft_usd_value(&ft);

        assert!(result.is_some());
        let val = result.unwrap();

        // 44 digits total minus 2 decimals = ends with 3 zeros (...456000)
        assert_eq!(
            val.tokens_scaled,
            "340282366920938463463374607431768211456000"
        );
        assert!(val.usd_value > 0.0);
    }

    #[test]
    fn test_padded_small_amounts() {
        // Case 1: Small balance where amount_str length < (decimals + 1)
        // 5 base units with 6 decimals -> 0.000005
        assert_eq!(
            parse_raw_amount_to_string("5", 6),
            Some("0.000005".to_string())
        );

        // Case 2: Small balance with trailing zeros
        // 500 base units with 6 decimals -> 0.000500 -> trimmed to 0.0005
        assert_eq!(
            parse_raw_amount_to_string("500", 6),
            Some("0.0005".to_string())
        );

        // Case 3: Edge case - zero balance
        assert_eq!(parse_raw_amount_to_string("0", 2), Some("0".to_string()));

        // Case 4: High decimals token (e.g., NEAR / ETH with 24 decimals)
        // 1000 base units with decimals = 24 -> 1000 / 10^24 = 10^-21 -> 0. (20 zeros) 1
        assert_eq!(
            parse_raw_amount_to_string("1000", 24),
            Some("0.000000000000000000001".to_string())
        );
    }

    #[test]
    fn test_negative_amount_returns_none() {
        // Case: API returns a negative number like "-100"
        // Rejected immediately during ASCII digit validation
        let ft = create_test_ft("-100", 6, Some(1.0));
        assert!(calculate_ft_usd_value(&ft).is_none());
    }

    #[test]
    fn test_invalid_amount_string() {
        // Case: API returns non-numeric characters
        let ft = create_test_ft("not_a_number", 6, Some(1.0));
        assert!(calculate_ft_usd_value(&ft).is_none());
    }

    #[test]
    fn test_missing_price() {
        // Case: price is missing (None)
        let ft = create_test_ft("5000000", 6, None);
        assert!(calculate_ft_usd_value(&ft).is_none());
    }

    #[test]
    fn test_zero_price() {
        // Case: price is present and equals $0.00
        let ft = create_test_ft("1000000", 6, Some(0.0));
        let result = calculate_ft_usd_value(&ft).unwrap();

        assert_eq!(result.tokens_scaled, "1");
        assert_eq!(result.usd_value, 0.0);
    }
}
