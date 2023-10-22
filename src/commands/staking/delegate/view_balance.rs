use color_eyre::eyre::WrapErr;

use crate::common::{CallResultExt, JsonRpcClientExt};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::DelegateStakeContext)]
#[interactive_clap(output_context = ViewBalanceContext)]
pub struct ViewBalance {
    #[interactive_clap(skip_default_input_arg)]
    /// On which account ID do you need to view the total balance?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct ViewBalanceContext(crate::network_view_at_block::ArgsForViewContext);

impl ViewBalanceContext {
    pub fn from_previous_context(
        previous_context: super::DelegateStakeContext,
        scope: &<ViewBalance as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let validator_account_id = previous_context.validator_account_id.clone();
        let interacting_with_account_ids = vec![validator_account_id.clone()];

        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let account_id: near_primitives::types::AccountId = scope.account_id.clone().into();

            move |network_config, block_reference| {
                let user_staked_balance: u128 = get_user_staked_balance(network_config, block_reference, &previous_context.validator_account_id, &account_id)?;
                let user_unstaked_balance: u128 = get_user_unstaked_balance(network_config, block_reference, &previous_context.validator_account_id, &account_id)?;
                let user_total_balance: u128 = get_user_total_balance(network_config, block_reference, &previous_context.validator_account_id, &account_id)?;

                eprintln!("Balance on validator <{validator_account_id}> for <{account_id}>:");
                eprintln!("      Staked balance:     {:>38}", crate::common::NearBalance::from_yoctonear(user_staked_balance).to_string());
                eprintln!("      Unstaked balance:   {:>38}", crate::common::NearBalance::from_yoctonear(user_unstaked_balance).to_string());
                eprintln!("      Total balance:      {:>38}", crate::common::NearBalance::from_yoctonear(user_total_balance).to_string());

                Ok(())
            }
        });
        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.global_context.config,
            interacting_with_account_ids,
            on_after_getting_block_reference_callback,
        }))
    }
}

impl From<ViewBalanceContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: ViewBalanceContext) -> Self {
        item.0
    }
}

impl ViewBalance {
    pub fn input_account_id(
        context: &super::DelegateStakeContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "On which account ID do you need to view the total balance?",
        )
    }
}

pub fn get_user_staked_balance(
    network_config: &crate::config::NetworkConfig,
    block_reference: &near_primitives::types::BlockReference,
    validator_account_id: &near_primitives::types::AccountId,
    account_id: &near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<u128> {
    Ok(network_config
        .json_rpc_client()
        .blocking_call_view_function(
            validator_account_id,
            "get_account_staked_balance",
            serde_json::to_vec(&serde_json::json!({
                "account_id": account_id,
            }))?,
            block_reference.clone(),
        )
        .wrap_err("Failed to fetch query for view method: 'get_account_staked_balance'")?
        .parse_result_from_json::<String>()
        .wrap_err("Failed to parse return value of view function call for String.")?
        .parse::<u128>()?)
}

pub fn get_user_unstaked_balance(
    network_config: &crate::config::NetworkConfig,
    block_reference: &near_primitives::types::BlockReference,
    validator_account_id: &near_primitives::types::AccountId,
    account_id: &near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<u128> {
    Ok(network_config
        .json_rpc_client()
        .blocking_call_view_function(
            validator_account_id,
            "get_account_unstaked_balance",
            serde_json::to_vec(&serde_json::json!({
                "account_id": account_id,
            }))?,
            block_reference.clone(),
        )
        .wrap_err("Failed to fetch query for view method: 'get_account_unstaked_balance'")?
        .parse_result_from_json::<String>()
        .wrap_err("Failed to parse return value of view function call for String.")?
        .parse::<u128>()?)
}

pub fn get_user_total_balance(
    network_config: &crate::config::NetworkConfig,
    block_reference: &near_primitives::types::BlockReference,
    validator_account_id: &near_primitives::types::AccountId,
    account_id: &near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<u128> {
    Ok(network_config
        .json_rpc_client()
        .blocking_call_view_function(
            validator_account_id,
            "get_account_total_balance",
            serde_json::to_vec(&serde_json::json!({
                "account_id": account_id,
            }))?,
            block_reference.clone(),
        )
        .wrap_err("Failed to fetch query for view method: 'get_account_total_balance'")?
        .parse_result_from_json::<String>()
        .wrap_err("Failed to parse return value of view function call for String.")?
        .parse::<u128>()?)
}
