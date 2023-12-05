use color_eyre::eyre::Context;
use serde_json::json;

use crate::common::CallResultExt;
use crate::common::JsonRpcClientExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::TokensCommandsContext)]
#[interactive_clap(output_context = ViewFtBalanceContext)]
pub struct ViewFtBalance {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the ft-contract account ID?
    ft_contract_account_id: crate::types::account_id::AccountId,
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
            let ft_contract_account_id: near_primitives::types::AccountId =
                scope.ft_contract_account_id.clone().into();

            move |network_config, block_reference| {
                let super::FtMetadata { decimals, symbol } = super::params_ft_metadata(
                    ft_contract_account_id.clone(),
                    network_config,
                    block_reference.clone(),
                )?;
                let args = serde_json::to_vec(&json!({
                    "account_id": owner_account_id.to_string(),
                    }))?;
                let call_result = network_config
                    .json_rpc_client()
                    .blocking_call_view_function(
                        &ft_contract_account_id,
                        "ft_balance_of",
                        args,
                        block_reference.clone(),
                    )
                    .wrap_err_with(||{
                        format!("Failed to fetch query for view method: 'ft_balance_of' (contract <{}> on network <{}>)",
                            ft_contract_account_id,
                            network_config.network_name
                        )
                    })?;
                call_result.print_logs();
                let amount: String = call_result.parse_result_from_json()?;
                let amount = amount.parse::<u128>().unwrap();
                let amount_fmt = {
                    if amount == 0 {
                        format!("0 {}", symbol)
                    } else if (amount % 10u128.pow(decimals as u32)) == 0 {
                        format!("{} {}", amount / 10u128.pow(decimals as u32), symbol,)
                    } else {
                        format!(
                            "{}.{} {}",
                            amount / 10u128.pow(decimals as u32),
                            format!("{:0>24}", amount % 10u128.pow(decimals as u32)).trim_end_matches('0'),
                            symbol
                        )
                    }
                };

                eprintln!(
                    "\n<{}> account has {}  (FT-contract: {})",
                    owner_account_id, amount_fmt, ft_contract_account_id
                );
                Ok(())
            }
        });
        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.global_context.config,
            on_after_getting_block_reference_callback,
            interacting_with_account_ids: vec![
                scope.ft_contract_account_id.clone().into(),
                previous_context.owner_account_id,
            ],
        }))
    }
}

impl From<ViewFtBalanceContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: ViewFtBalanceContext) -> Self {
        item.0
    }
}

impl ViewFtBalance {
    pub fn input_ft_contract_account_id(
        context: &super::TokensCommandsContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the ft-contract account ID?",
        )
    }
}
