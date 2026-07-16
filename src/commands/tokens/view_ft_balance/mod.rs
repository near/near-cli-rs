use color_eyre::eyre::Context;
use serde_json::json;

use crate::common::CallResultExt;
use crate::common::JsonRpcClientExt;

use super::send_ft::input_ft_contract_account_id;
use crate::types::ft_nft_inventory::{FTContract, FTInventory, get_account_ft_nft_token_inventory};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::TokensCommandsContext)]
#[interactive_clap(output_context = ViewFtBalanceContext)]
pub struct ViewFtBalance {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the ft-contract account ID?
    ft_contract: crate::types::ft_nft_inventory::FTContract,
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
                    print_fts_inventory(network_config, &owner_account_id)?;
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

fn print_fts_inventory(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
) -> crate::CliResult {
    let inventory = get_account_ft_nft_token_inventory(network_config, account_id)?;
    let mut fts = inventory.fts();

    if fts.is_empty() {
        println!(
            "<{account_id}> account has no fungible tokens (FTs) on network <{}>.",
            network_config.network_name
        );
        return Ok(());
    }

    fn ft_value(ft: &FTInventory) -> u128 {
        let amount = ft.amount.parse::<u128>().unwrap_or_default();
        let decimals = ft.ft_meta.decimals as u32;

        const PRICE_PRECISION_U128: u128 = 1_000_000_000u128; // 1e9

        let one_ft = 10u128.checked_pow(decimals).unwrap_or(u128::MAX);
        // If decimals overflow, use MAX so the computed value rounds down toward 0.

        // Convert price to a scaled integer and use checked integer arithmetic
        // to avoid panic on overflow during multiplication/division.
        let price = match ft.ft_meta.price {
            Some(p) if p.is_finite() && p >= 0.0 => p,
            _ => return 0u128,
        };

        let price_scaled = (price * (PRICE_PRECISION_U128 as f64)).round();
        if price_scaled <= 0.0 {
            return 0u128;
        }

        let price_scaled_u128 = price_scaled as u128;
        amount
            .checked_mul(price_scaled_u128)
            .and_then(|prod| prod.checked_div(one_ft))
            .unwrap_or(0u128)
    }

    fts.sort_by(|a, b| {
        let a_val = ft_value(a);
        let b_val = ft_value(b);
        b_val.cmp(&a_val)
    });
    let fts = fts
        .iter()
        .map(|ft_inventory| {
            let fungible_token = crate::types::ft_properties::FungibleToken::from_params_ft(
                ft_inventory.amount.parse::<u128>().unwrap_or_default(),
                ft_inventory.ft_meta.decimals,
                ft_inventory.ft_meta.symbol.clone(),
            );
            format!(
                "\t{} (FT-contract: {})\n",
                fungible_token, ft_inventory.ft_contract_account_id
            )
        })
        .collect::<String>();
    println!("<{account_id}> account has:\n{fts}");
    Ok(())
}
