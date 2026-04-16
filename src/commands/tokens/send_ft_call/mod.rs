use color_eyre::eyre::{Context, ContextCompat};
use serde_json::{Value, json};

use crate::common::CallResultExt;
use crate::common::{blocking_view_function, to_call_result};

use super::send_ft::input_ft_contract_account_id;

mod amount_ft;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::TokensCommandsContext)]
#[interactive_clap(output_context = FtContractContext)]
pub struct FtContract {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the ft-contract account ID?
    ft_contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Specify sending FTCall command parameters:
    send_ft_call_command: SendFtCallCommand,
}

#[derive(Debug, Clone)]
pub struct FtContractContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    ft_contract: crate::types::ft_properties::FtContract,
}

impl FtContractContext {
    pub fn from_previous_context(
        previous_context: super::TokensCommandsContext,
        scope: &<FtContract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let ft_contract_account_id: near_primitives::types::AccountId =
            scope.ft_contract_account_id.clone().into();

        let ft_metadata = {
            let network_config = crate::common::find_network_where_account_exist(
                &previous_context.global_context,
                ft_contract_account_id.clone(),
            )?
            .wrap_err_with(|| {
                format!(
                    "Contract <{}> does not exist in networks",
                    ft_contract_account_id
                )
            })?;

            crate::types::ft_properties::params_ft_metadata(
                ft_contract_account_id.clone(),
                &network_config,
                near_primitives::types::Finality::Final.into(),
            )?
        };

        let ft_contract = crate::types::ft_properties::FtContract {
            ft_metadata,
            ft_contract_account_id,
        };

        crate::common::update_used_ft_contract_account_list(
            &previous_context.global_context.config.credentials_home_dir,
            &ft_contract,
        );

        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.owner_account_id,
            ft_contract,
        })
    }
}

impl FtContract {
    pub fn input_ft_contract_account_id(
        context: &super::TokensCommandsContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        input_ft_contract_account_id(&context.global_context.config.credentials_home_dir)
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = FtContractContext)]
#[interactive_clap(output_context = SendFtCallCommandContext)]
pub struct SendFtCallCommand {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the receiver account ID?
    receiver_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subargs)]
    /// Specify amount FT
    amount_ft: self::amount_ft::AmountFt,
}

#[derive(Debug, Clone)]
pub struct SendFtCallCommandContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    ft_contract: crate::types::ft_properties::FtContract,
    receiver_account_id: near_primitives::types::AccountId,
}

impl SendFtCallCommandContext {
    pub fn from_previous_context(
        previous_context: FtContractContext,
        scope: &<SendFtCallCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            ft_contract: previous_context.ft_contract,
            receiver_account_id: scope.receiver_account_id.clone().into(),
        })
    }
}

impl SendFtCallCommand {
    pub fn input_receiver_account_id(
        context: &FtContractContext,
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
    memo: &str,
    msg: &str,
    deposit: crate::types::near_token::NearToken,
    gas: crate::common::NearGas,
) -> color_eyre::eyre::Result<crate::commands::PrepopulatedTransaction> {
    tracing::info!(target: "near_teach_me", "Creating a pre-populated transaction for signature ...");
    let args_ft_transfer_call = serde_json::to_vec(&crate::types::ft_properties::FtTransferCall {
        receiver_id: receiver_account_id.clone(),
        amount: amount_ft.amount(),
        memo: if memo.is_empty() {
            None
        } else {
            Some(memo.to_string())
        },
        msg: msg.to_string(),
    })?;

    let action_ft_transfer_call = near_kit::Action::FunctionCall(near_kit::FunctionCallAction {
        method_name: "ft_transfer_call".to_string(),
        args: args_ft_transfer_call,
        gas: near_kit::Gas::from_gas(gas.as_gas()),
        deposit: deposit.into(),
    });

    let args = serde_json::to_vec(&json!({"account_id": receiver_account_id}))?;

    let call_result = to_call_result(&blocking_view_function(
            network_config,
            ft_contract_account_id,
            "storage_balance_of",
            args.clone(),
            near_primitives::types::Finality::Final.into(),
        )
        .wrap_err_with(|| {
            format!(
                "Failed to fetch query for view method: 'storage_balance_of' (contract <{}> on network <{}>)",
                ft_contract_account_id, network_config.network_name
            )
        })?);

    if call_result.parse_result_from_json::<Value>()?.is_null() {
        let action_storage_deposit = near_kit::Action::FunctionCall(near_kit::FunctionCallAction {
            method_name: "storage_deposit".to_string(),
            args,
            gas: near_kit::Gas::from_gas(gas.as_gas()),
            deposit: near_token::NearToken::from_millinear(100),
        });
        return Ok(crate::commands::PrepopulatedTransaction {
            signer_id: signer_id.clone(),
            receiver_id: ft_contract_account_id.clone(),
            actions: vec![action_storage_deposit, action_ft_transfer_call.clone()],
        });
    }

    Ok(crate::commands::PrepopulatedTransaction {
        signer_id: signer_id.clone(),
        receiver_id: ft_contract_account_id.clone(),
        actions: vec![action_ft_transfer_call.clone()],
    })
}
