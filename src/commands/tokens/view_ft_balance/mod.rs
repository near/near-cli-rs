use color_eyre::eyre::Context;
use rust_decimal::{Decimal, prelude::FromPrimitive};
use serde_json::json;

use crate::common::CallResultExt;
use crate::common::JsonRpcClientExt;

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
                    let ft_contract_account_id: near_primitives::types::AccountId = ft_contract_account_id.clone().into();
                    let ft_metadata = crate::types::ft_properties::params_ft_metadata(
                        ft_contract_account_id.clone(),
                        network_config,
                        block_reference.clone(),
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
                    let call_result = get_ft_balance(network_config, &ft_contract_account_id, args, block_reference.clone())?;
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
    ft_contract_account_id: &near_primitives::types::AccountId,
    args: Vec<u8>,
    block_reference: near_primitives::types::BlockReference,
) -> color_eyre::eyre::Result<near_primitives::views::CallResult> {
    tracing::info!(target: "near_teach_me", "Getting FT balance ...");
    network_config
        .json_rpc_client()
        .blocking_call_view_function(
            ft_contract_account_id,
            "ft_balance_of",
            args,
            block_reference,
        )
        .wrap_err_with(||{
            format!("Failed to fetch query for view method: 'ft_balance_of' (contract <{}> on network <{}>)",
                ft_contract_account_id,
                network_config.network_name
            )
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FTCalculatedValue {
    ft: FTInventory,
    tokens_scaled: Decimal,
    usd_value: Decimal,
}

fn calculate_ft_usd_value(ft: &FTInventory) -> Option<FTCalculatedValue> {
    let tokens = ft.amount.parse::<u128>().ok()?;
    let price = ft.ft_meta.price?;
    let decimals = ft.ft_meta.decimals as u32;

    let tokens_decimal = Decimal::from_u128(tokens)?;
    let divisor = Decimal::from_u128(10u128.checked_pow(decimals)?)?;
    let tokens_scaled = tokens_decimal.checked_div(divisor)?;
    let usd_value = tokens_scaled.checked_mul(price)?;

    Some(FTCalculatedValue {
        ft: ft.clone(),
        tokens_scaled,
        usd_value,
    })
}

fn print_fts_inventory(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
    verbosity: crate::Verbosity,
) -> crate::CliResult {
    let inventory = get_account_ft_inventory(network_config, account_id)?;
    let min_usd = Decimal::new(10, 2); // $0.10

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

    fts.sort_by_key(|a| std::cmp::Reverse(a.usd_value));

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
    fn create_test_ft(amount: &str, decimals: u8, price: Option<Decimal>) -> FTInventory {
        FTInventory {
            amount: amount.to_string(),
            ft_contract_account_id: near_primitives::types::AccountId::from_str("test.near")
                .unwrap(),
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
        // 10050 base units / 10^2 = 100.50 tokens. 100.50 * 2.50 = 251.25
        let ft = create_test_ft("10050", 2, Some(Decimal::new(250, 2)));
        let result = calculate_ft_usd_value(&ft);

        let expected = FTCalculatedValue {
            ft: ft.clone(),
            tokens_scaled: Decimal::new(10050, 2), // 100.50
            usd_value: Decimal::new(25125, 2),     // 251.25
        };

        assert_eq!(result, Some(expected));
    }

    #[test]
    fn test_missing_price() {
        // Case: price is missing (None).
        // The function should safely return None
        let ft = create_test_ft("5000000", 6, None);
        let result = calculate_ft_usd_value(&ft);

        assert_eq!(result, None);
    }

    #[test]
    fn test_zero_price() {
        // Case: price is present and equals $0.00 (Some(0)).
        // Should calculate normally and return $0.00 value
        let ft = create_test_ft("1000000", 6, Some(Decimal::ZERO));
        let result = calculate_ft_usd_value(&ft);

        let expected = FTCalculatedValue {
            ft: ft.clone(),
            tokens_scaled: Decimal::ONE,
            usd_value: Decimal::ZERO,
        };
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn test_amount_parsing_overflow() {
        // Case: amount string contains a number larger than u128::MAX
        // The string won't parse due to parse::<u128>().ok()?, returning None
        let crazy_amount = "34028236692093846346337460743176821145600000"; // u128::MAX + extra zeroes
        let ft = create_test_ft(crazy_amount, 6, Some(Decimal::new(55, 1)));
        let result = calculate_ft_usd_value(&ft);

        assert_eq!(result, None);
    }

    #[test]
    fn test_invalid_amount_string() {
        // Case: amount contains an invalid string or negative value
        let ft = create_test_ft("not_a_number", 6, Some(Decimal::ONE));
        let result = calculate_ft_usd_value(&ft);

        assert_eq!(result, None);
    }

    #[test]
    fn test_decimals_overflow_checked_pow() {
        // Case: decimals value is too large (40), breaking 10u128.checked_pow(40)
        let ft = create_test_ft("1000000", 40, Some(Decimal::new(1, 0)));
        let result = calculate_ft_usd_value(&ft);

        assert_eq!(result, None);
    }

    #[test]
    fn test_decimal_type_overflow_by_divisor() {
        // Case: decimals = 30. 10^30 exceeds the limits of Decimal (28 digits)
        let ft = create_test_ft("1000000", 30, Some(Decimal::new(1, 0)));
        let result = calculate_ft_usd_value(&ft);

        assert_eq!(result, None);
    }

    #[test]
    fn test_multiplication_overflow() {
        // Case: Multiplication result exceeds Decimal::MAX
        let ft = create_test_ft(
            "79228162514264337593543950335",
            0,
            Some(Decimal::new(10, 0)),
        );
        let result = calculate_ft_usd_value(&ft);

        assert_eq!(result, None);
    }
}
