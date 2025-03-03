use color_eyre::eyre::WrapErr;

use crate::common::{CallResultExt, JsonRpcClientExt};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::StakeDelegationContext)]
#[interactive_clap(output_context = ViewBalanceContext)]
pub struct ViewBalance {
    #[interactive_clap(skip_default_input_arg)]
    /// What is validator account ID?
    validator_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct ViewBalanceContext(crate::network_view_at_block::ArgsForViewContext);

impl ViewBalanceContext {
    pub fn from_previous_context(
        previous_context: super::StakeDelegationContext,
        scope: &<ViewBalance as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let account_id = previous_context.account_id.clone();
        let validator_account_id: near_primitives::types::AccountId =
            scope.validator_account_id.clone().into();
        let interacting_with_account_ids = vec![account_id.clone(), validator_account_id.clone()];

        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({

            move |network_config: &crate::config::NetworkConfig, block_reference: &near_primitives::types::BlockReference| {
                calculation_delegated_stake_balance(
                    &account_id,
                    &validator_account_id,
                    network_config,
                    block_reference,
                    &previous_context.global_context.verbosity
                )
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
    pub fn input_validator_account_id(
        context: &super::StakeDelegationContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_staking_pool_validator_account_id(&context.global_context.config)
    }
}

#[tracing::instrument(
    name = "Calculation of the delegated stake balance for your account ...",
    skip_all
)]
fn calculation_delegated_stake_balance(
    account_id: &near_primitives::types::AccountId,
    validator_account_id: &near_primitives::types::AccountId,
    network_config: &crate::config::NetworkConfig,
    block_reference: &near_primitives::types::BlockReference,
    verbosity: &crate::Verbosity,
) -> crate::CliResult {
    let user_staked_balance: u128 = get_user_staked_balance(
        network_config,
        block_reference,
        validator_account_id,
        account_id,
    )?;
    let user_unstaked_balance: u128 = get_user_unstaked_balance(
        network_config,
        block_reference,
        validator_account_id,
        account_id,
    )?;
    let user_total_balance: u128 = get_user_total_balance(
        network_config,
        block_reference,
        validator_account_id,
        account_id,
    )?;
    let withdrawal_availability_message =
        match is_account_unstaked_balance_available_for_withdrawal(
            network_config,
            validator_account_id,
            account_id,
        )? {
            true if user_unstaked_balance > 0 => "(available for withdrawal)",
            false if user_unstaked_balance > 0 => {
                "(not available for withdrawal in the current epoch)"
            }
            _ => "",
        };

    let mut info_str = String::new();
    info_str.push_str(&format!(
        "\n      Staked balance:     {:>38}",
        near_token::NearToken::from_yoctonear(user_staked_balance).to_string()
    ));
    info_str.push_str(&format!(
        "\n      Unstaked balance:   {:>38} {withdrawal_availability_message}",
        near_token::NearToken::from_yoctonear(user_unstaked_balance).to_string()
    ));
    info_str.push_str(&format!(
        "\n      Total balance:      {:>38}",
        near_token::NearToken::from_yoctonear(user_total_balance).to_string()
    ));
    tracing::info!(
        parent: &tracing::Span::none(),
        "{}{}",
        format!("Delegated stake balance with validator <{validator_account_id}> by <{account_id}>:"),
        crate::common::indent_payload(&info_str)
    );
    if let crate::Verbosity::Quiet = verbosity {
        println!(
            "Delegated stake balance with validator <{validator_account_id}> by <{account_id}>:{}",
            crate::common::indent_payload(&info_str)
        );
    };
    Ok(())
}

#[tracing::instrument(name = "Getting the staked balance for the user ...", skip_all)]
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
        .wrap_err_with(||{
            format!("Failed to fetch query for view method: 'get_account_staked_balance' (contract <{}> on network <{}>)",
                validator_account_id,
                network_config.network_name
            )
        })?
        .parse_result_from_json::<String>()
        .wrap_err("Failed to parse return value of view function call for String.")?
        .parse::<u128>()?)
}

#[tracing::instrument(name = "Getting the unstaked balance for the user ...", skip_all)]
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
        .wrap_err_with(||{
            format!("Failed to fetch query for view method: 'get_account_unstaked_balance' (contract <{}> on network <{}>)",
                validator_account_id,
                network_config.network_name
            )
        })?
        .parse_result_from_json::<String>()
        .wrap_err("Failed to parse return value of view function call for String.")?
        .parse::<u128>()?)
}

#[tracing::instrument(name = "Getting the total balance for the user ...", skip_all)]
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
        .wrap_err_with(||{
            format!("Failed to fetch query for view method: 'get_account_total_balance' (contract <{}> on network <{}>)",
                validator_account_id,
                network_config.network_name
            )
        })?
        .parse_result_from_json::<String>()
        .wrap_err("Failed to parse return value of view function call for String.")?
        .parse::<u128>()?)
}

#[tracing::instrument(
    name = "Getting account unstaked balance available for withdrawal ...",
    skip_all
)]
pub fn is_account_unstaked_balance_available_for_withdrawal(
    network_config: &crate::config::NetworkConfig,
    validator_account_id: &near_primitives::types::AccountId,
    account_id: &near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<bool> {
    network_config
        .json_rpc_client()
        .blocking_call_view_function(
            validator_account_id,
            "is_account_unstaked_balance_available",
            serde_json::to_vec(&serde_json::json!({
                "account_id": account_id.to_string(),
            }))?,
            near_primitives::types::BlockReference::Finality(
                near_primitives::types::Finality::Final,
            ),
        )
        .wrap_err_with(||{
            format!("Failed to fetch query for view method: 'is_account_unstaked_balance_available' (contract <{}> on network <{}>)",
                validator_account_id,
                network_config.network_name
            )
        })?
        .parse_result_from_json::<bool>()
        .wrap_err("Failed to parse return value of view function call for bool value.")
}
