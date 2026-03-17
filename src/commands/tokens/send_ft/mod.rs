use std::str::FromStr;

use color_eyre::eyre::{Context, ContextCompat};
use serde_json::{Value, json};

use crate::common::CallResultExt;
use crate::common::JsonRpcClientExt;

use super::view_ft_balance::get_ft_balance;

mod amount_ft;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::TokensCommandsContext)]
#[interactive_clap(output_context = FtContractContext)]
pub struct FtContract {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the ft-contract account ID?
    ft_contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Specify sending FT command parameters:
    send_ft_command: SendFtCommand,
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

        let hash_map: std::collections::HashMap<
            near_primitives::types::AccountId,
            crate::types::ft_properties::FtMetadata,
        > = crate::common::get_used_ft_contract_account_list(
            &previous_context.global_context.config.credentials_home_dir,
        )
        .into_iter()
        .map(|ft_contract| {
            (
                ft_contract.ft_contract_account_id.clone(),
                ft_contract.ft_metadata.clone(),
            )
        })
        .collect();

        let ft_metadata = if let Some(ft_metadata) = hash_map.get(&ft_contract_account_id) {
            ft_metadata.clone()
        } else {
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
        let used_ft_contract_account_list = crate::common::get_used_ft_contract_account_list(
            &context.global_context.config.credentials_home_dir,
        )
        .into_iter()
        .map(|ft_contract_account| {
            format!(
                "{} ({})",
                ft_contract_account.ft_metadata.symbol, ft_contract_account.ft_contract_account_id
            )
        })
        .collect::<Vec<_>>();
        let account_id_str = match inquire::Text::new(
            "Select from the list or enter a different ft-contract account ID:",
        )
        .with_autocomplete(move |val: &str| {
            Ok(used_ft_contract_account_list
                .iter()
                .filter(|s| s.to_lowercase().contains(&val.to_lowercase()))
                .cloned()
                .collect())
        })
        .with_validator(|ft_contract_account_str: &str| {
            let account_id_str =
                &get_account_id_str_from_ft_contract_account_str(ft_contract_account_str);

            match near_primitives::types::AccountId::validate(account_id_str) {
                Ok(_) => Ok(inquire::validator::Validation::Valid),
                Err(err) => Ok(inquire::validator::Validation::Invalid(
                    inquire::validator::ErrorMessage::Custom(format!("Invalid account ID: {err}")),
                )),
            }
        })
        .prompt()
        {
            Ok(value) => get_account_id_str_from_ft_contract_account_str(&value),
            Err(
                inquire::error::InquireError::OperationCanceled
                | inquire::error::InquireError::OperationInterrupted,
            ) => return Ok(None),
            Err(err) => return Err(err.into()),
        };
        let account_id = crate::types::account_id::AccountId::from_str(&account_id_str)?;

        Ok(Some(account_id))
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = FtContractContext)]
#[interactive_clap(output_context = SendFtCommandContext)]
pub struct SendFtCommand {
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
    ft_contract: crate::types::ft_properties::FtContract,
    receiver_account_id: near_primitives::types::AccountId,
}

impl SendFtCommandContext {
    pub fn from_previous_context(
        previous_context: FtContractContext,
        scope: &<SendFtCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            ft_contract: previous_context.ft_contract,
            receiver_account_id: scope.receiver_account_id.clone().into(),
        })
    }
}

impl SendFtCommand {
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
    deposit: crate::types::near_token::NearToken,
    gas: crate::common::NearGas,
) -> color_eyre::eyre::Result<crate::commands::PrepopulatedTransaction> {
    tracing::info!(target: "near_teach_me", "Creating a pre-populated transaction for signature ...");
    let args_ft_transfer = serde_json::to_vec(&crate::types::ft_properties::FtTransfer {
        receiver_id: receiver_account_id.clone(),
        amount: amount_ft.amount(),
        memo: if memo.is_empty() {
            None
        } else {
            Some(memo.to_string())
        },
    })?;

    let action_ft_transfer = near_primitives::transaction::Action::FunctionCall(Box::new(
        near_primitives::transaction::FunctionCallAction {
            method_name: "ft_transfer".to_string(),
            args: args_ft_transfer,
            gas: near_primitives::gas::Gas::from_gas(gas.as_gas()),
            deposit: deposit.into(),
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
                gas: near_primitives::gas::Gas::from_gas(gas.as_gas()),
                deposit: near_token::NearToken::from_millinear(100),
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

fn get_account_id_str_from_ft_contract_account_str(ft_contract_account_str: &str) -> String {
    ft_contract_account_str
        .split_whitespace()
        .last()
        .unwrap_or_default()
        .trim_matches(|c| c == '(' || c == ')')
        .to_string()
}
