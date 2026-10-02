use futures::StreamExt;

use crate::commands::tokens::view_ft_balance::parse_raw_amount_to_string;
use crate::types::mt_ft_inventory::{get_mt_ft_inventory, get_mt_tokens_for_owner};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::TokensCommandsContext)]
#[interactive_clap(output_context = ViewMtFtBalanceContext)]
pub struct ViewMtFtBalance {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the mt-contract account ID?
    mt_contract: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct ViewMtFtBalanceContext(crate::network_view_at_block::ArgsForViewContext);

impl ViewMtFtBalanceContext {
    pub fn from_previous_context(
        previous_context: super::TokensCommandsContext,
        scope: &<ViewMtFtBalance as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let mt_contract: near_primitives::types::AccountId = scope.mt_contract.clone().into();
            let owner_account_id = previous_context.owner_account_id.clone();

            move |network_config, block_reference| {
                display_mt_fts_inventory(network_config, &mt_contract, &owner_account_id, block_reference, previous_context.global_context.verbosity)
            }
        });
        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.global_context.config,
            on_after_getting_block_reference_callback,
            interacting_with_account_ids: vec![
                previous_context.owner_account_id.clone(),
                scope.mt_contract.clone().into(),
            ],
        }))
    }
}

impl From<ViewMtFtBalanceContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: ViewMtFtBalanceContext) -> Self {
        item.0
    }
}

impl ViewMtFtBalance {
    pub fn input_mt_contract(
        context: &super::TokensCommandsContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the account ID of a contract that supports the multi-token (MT) standard for a fungible token (FT)?",
        )
    }
}

fn handle_mt_ft_inventory_result(
    owner_account_id: &near_primitives::types::AccountId,
    ft: &crate::types::mt_ft_inventory::FT,
    result: color_eyre::eyre::Result<crate::types::mt_ft_inventory::MtFtInventory>,
) -> Option<crate::types::mt_ft_inventory::MtFtInventory> {
    match result {
        Ok(inventory) => Some(inventory),
        Err(error) => {
            tracing::warn!(
                "Skipping MT-FT asset <{}> for account <{owner_account_id}>: {error}",
                ft.token_id
            );
            None
        }
    }
}

fn display_mt_fts_inventory(
    network_config: &crate::config::NetworkConfig,
    mt_contract: &near_primitives::types::AccountId,
    owner_account_id: &near_primitives::types::AccountId,
    block_reference: &near_primitives::types::BlockReference,
    verbosity: crate::Verbosity,
) -> crate::CliResult {
    let mt_fts = get_mt_tokens_for_owner(
        network_config,
        mt_contract,
        owner_account_id,
        block_reference.clone(),
    )?;

    if mt_fts.is_empty() {
        if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe = verbosity {
            eprintln!(
                "The account <{owner_account_id}> has no fungible tokens of multi-token type <{mt_contract}> on network <{}>.",
                network_config.network_name
            );
        }
        return Ok(());
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let concurrency = 10;

    let mut mt_ft_inventories: Vec<_> = runtime
        .block_on(async {
            futures::stream::iter(mt_fts)
                .map(|ft| async move {
                    let result = get_mt_ft_inventory(
                        network_config,
                        &mt_contract.clone(),
                        &ft.token_id,
                        &owner_account_id.clone(),
                        block_reference.clone(),
                    )
                    .await;
                    handle_mt_ft_inventory_result(owner_account_id, &ft, result)
                })
                .buffer_unordered(concurrency)
                .collect::<Vec<_>>()
                .await
        })
        .into_iter()
        .flatten()
        .collect();

    if mt_ft_inventories.is_empty() {
        if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe = verbosity {
            eprintln!(
                "The account <{owner_account_id}> has no readable fungible tokens of multi-token type <{mt_contract}> on network <{}>.",
                network_config.network_name
            );
        }
        return Ok(());
    }

    // Sort by token symbol
    mt_ft_inventories.sort_by(|a, b| a.ft_token.symbol().cmp(b.ft_token.symbol()));

    eprintln!(
        "\nThe account <{owner_account_id}> has fungible tokens of multi-token type <{mt_contract}>:"
    );

    let mut table = prettytable::Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_NO_COLSEP);

    for token in mt_ft_inventories {
        table.add_row(prettytable::row![
            Fy->token.ft_token.symbol(),
            parse_raw_amount_to_string(&token.ft_token.amount().to_string(), token.ft_token.decimals()).unwrap_or_default(),
            token.token_id.to_string(),
        ]);
    }

    table.printstd();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_handle_mt_ft_inventory_result_keeps_valid_inventory() {
        let owner_account_id = near_primitives::types::AccountId::from_str("alice.near").unwrap();
        let ft = crate::types::mt_ft_inventory::FT {
            token_id: crate::types::mt_ft_inventory::IntentsTokenId::from_str("nep141:wrap.near")
                .unwrap(),
        };
        let inventory = crate::types::mt_ft_inventory::MtFtInventory {
            token_id: ft.token_id.clone(),
            ft_token: crate::types::ft_properties::FungibleToken::from_params_ft(
                42,
                4,
                "wNEAR".to_string(),
            ),
            price: None,
        };

        let result = handle_mt_ft_inventory_result(&owner_account_id, &ft, Ok(inventory.clone()));

        assert_eq!(result, Some(inventory));
    }

    #[test]
    fn test_handle_mt_ft_inventory_result_skips_failed_inventory() {
        let owner_account_id = near_primitives::types::AccountId::from_str("alice.near").unwrap();
        let ft = crate::types::mt_ft_inventory::FT {
            token_id: crate::types::mt_ft_inventory::IntentsTokenId::from_str(
                "nep245:v2.omni.near:token",
            )
            .unwrap(),
        };

        let result = handle_mt_ft_inventory_result(
            &owner_account_id,
            &ft,
            Err(color_eyre::eyre::eyre!("metadata lookup failed")),
        );

        assert_eq!(result, None);
    }
}
