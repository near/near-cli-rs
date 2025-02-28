use color_eyre::eyre::Context;
use serde_json::{json, Value};

use crate::common::CallResultExt;
use crate::common::JsonRpcClientExt;

use super::view_ft_balance::get_ft_balance;

mod amount_ft;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::TokensCommandsContext)]
#[interactive_clap(output_context = SendFtCommandContext)]
pub struct SendFtCommand {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the ft-contract account ID?
    ft_contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(skip_default_input_arg)]
    /// What is the receiver account ID?
    receiver_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subargs)]
    /// Specify amount FT
    amount_ft: self::amount_ft::AmountFt,
}

#[derive(Debug, Clone)]
pub struct SendFtCommandContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    ft_contract_account_id: near_primitives::types::AccountId,
    receiver_account_id: near_primitives::types::AccountId,
}

impl SendFtCommandContext {
    pub fn from_previous_context(
        previous_context: super::TokensCommandsContext,
        scope: &<SendFtCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.owner_account_id,
            ft_contract_account_id: scope.ft_contract_account_id.clone().into(),
            receiver_account_id: scope.receiver_account_id.clone().into(),
        })
    }
}

impl SendFtCommand {
    pub fn input_ft_contract_account_id(
        context: &super::TokensCommandsContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the ft-contract account ID?",
        )
    }

    pub fn input_receiver_account_id(
        context: &super::TokensCommandsContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the receiver account ID?",
        )
    }
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(
    name = "Creating a pre-populated transaction for signature ...",
    skip_all
)]
pub fn get_prepopulated_transaction(
    network_config: &crate::config::NetworkConfig,
    ft_contract_account_id: &near_primitives::types::AccountId,
    receiver_account_id: &near_primitives::types::AccountId,
    signer_id: &near_primitives::types::AccountId,
    amount_ft: &crate::types::ft_properties::FungibleToken,
    memo: &Option<String>,
    deposit: &crate::types::near_token::NearToken,
    gas: &crate::common::NearGas,
) -> color_eyre::eyre::Result<crate::commands::PrepopulatedTransaction> {
    let args_ft_transfer = serde_json::to_vec(&json!({
        "receiver_id": receiver_account_id,
        "amount": amount_ft.amount().to_string(),
        "memo": memo.as_ref().and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
    }))?;

    let action_ft_transfer = near_primitives::transaction::Action::FunctionCall(Box::new(
        near_primitives::transaction::FunctionCallAction {
            method_name: "ft_transfer".to_string(),
            args: args_ft_transfer,
            gas: gas.as_gas(),
            deposit: deposit.as_yoctonear(),
        },
    ));

    let args = serde_json::to_vec(&json!({"account_id": receiver_account_id}))?;

    let call_result = network_config
        .json_rpc_client()
        .blocking_call_view_function(
            ft_contract_account_id,
            "storage_balance_of",
            args.clone(),
            near_primitives::types::Finality::Final.into(),
        )
        .wrap_err_with(||{
            format!("Failed to fetch query for view method: 'storage_balance_of' (contract <{}> on network <{}>)",
                ft_contract_account_id,
                network_config.network_name
            )
        })?;

    if call_result.parse_result_from_json::<Value>()?.is_null() {
        let action_storage_deposit = near_primitives::transaction::Action::FunctionCall(Box::new(
            near_primitives::transaction::FunctionCallAction {
                method_name: "storage_deposit".to_string(),
                args,
                gas: gas.as_gas(),
                deposit: near_token::NearToken::from_millinear(100).as_yoctonear(),
            },
        ));
        return Ok(crate::commands::PrepopulatedTransaction {
            signer_id: signer_id.clone(),
            receiver_id: ft_contract_account_id.clone(),
            actions: vec![action_storage_deposit, action_ft_transfer.clone()],
        });
    }

    Ok(crate::commands::PrepopulatedTransaction {
        signer_id: signer_id.clone(),
        receiver_id: ft_contract_account_id.clone(),
        actions: vec![action_ft_transfer.clone()],
    })
}

fn get_ft_balance_for_account(
    network_config: &crate::config::NetworkConfig,
    signer_account_id: &near_primitives::types::AccountId,
    ft_contract_account_id: &near_primitives::types::AccountId,
    block_reference: near_primitives::types::BlockReference,
) -> color_eyre::eyre::Result<crate::types::ft_properties::FungibleToken> {
    let function_args = serde_json::to_vec(&json!({"account_id": signer_account_id}))?;
    let amount = get_ft_balance(
        network_config,
        ft_contract_account_id,
        function_args,
        block_reference,
    )?
    .parse_result_from_json::<String>()?;
    let crate::types::ft_properties::FtMetadata { decimals, symbol } =
        crate::types::ft_properties::params_ft_metadata(
            ft_contract_account_id.clone(),
            network_config,
            near_primitives::types::Finality::Final.into(),
        )?;
    Ok(crate::types::ft_properties::FungibleToken::from_params_ft(
        amount.parse::<u128>()?,
        decimals,
        symbol,
    ))
}
