use std::str::FromStr;

use color_eyre::{
    eyre::{Context, ContextCompat},
    owo_colors::OwoColorize,
};
use inquire::{CustomType, Text};
use serde_json::{json, Value};

use crate::common::CallResultExt;
use crate::common::JsonRpcClientExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::SendFtCommandContext)]
#[interactive_clap(output_context = AmountFtContext)]
pub struct AmountFt {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter an amount FT to transfer:
    amount_ft: crate::types::ft_properties::FungibleToken,
    #[interactive_clap(named_arg)]
    /// Enter gas for function call
    prepaid_gas: PrepaidGas,
}

#[derive(Debug, Clone)]
pub struct AmountFtContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    ft_contract_account_id: near_primitives::types::AccountId,
    receiver_account_id: near_primitives::types::AccountId,
    amount_ft: crate::types::ft_properties::FungibleToken,
}

impl AmountFtContext {
    pub fn from_previous_context(
        previous_context: super::SendFtCommandContext,
        scope: &<AmountFt as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let network_config = crate::common::find_network_where_account_exist(
            &previous_context.global_context,
            previous_context.ft_contract_account_id.clone(),
        )
        .wrap_err_with(|| {
            format!(
                "Contract <{}> does not exist in networks",
                previous_context.ft_contract_account_id
            )
        })?;
        let crate::types::ft_properties::FtMetadata { decimals, symbol } =
            crate::types::ft_properties::params_ft_metadata(
                previous_context.ft_contract_account_id.clone(),
                &network_config,
                near_primitives::types::Finality::Final.into(),
            )?;
        let mut amount_ft = scope.amount_ft.clone();

        if amount_ft.symbol.to_uppercase() != symbol.to_uppercase() {
            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "Invalid currency symbol"
            ));
        } else if amount_ft.decimals > decimals as u8 {
            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "Error: Invalid decimal places. Your FT amount exceeds {decimals} decimal places."
            ));
        } else {
            amount_ft.amount = amount_ft
                .amount
                .checked_mul(10u128.pow(decimals as u32 - amount_ft.decimals as u32))
                .wrap_err("FT Balance: underflow or overflow happens")?;
            amount_ft.decimals = decimals as u8;
            amount_ft.symbol = symbol;
        }

        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            ft_contract_account_id: previous_context.ft_contract_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            amount_ft,
        })
    }
}

impl AmountFt {
    fn input_amount_ft(
        context: &super::SendFtCommandContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::ft_properties::FungibleToken>> {
        let network_config = crate::common::find_network_where_account_exist(
            &context.global_context,
            context.ft_contract_account_id.clone(),
        )
        .wrap_err_with(|| {
            format!(
                "Contract <{}> does not exist in networks",
                context.ft_contract_account_id
            )
        })?;

        let crate::types::ft_properties::FtMetadata { decimals, symbol } =
            crate::types::ft_properties::params_ft_metadata(
                context.ft_contract_account_id.clone(),
                &network_config,
                near_primitives::types::Finality::Final.into(),
            )?;
        eprintln!();
        loop {
            match CustomType::<crate::types::ft_properties::FungibleToken>::new(&format!(
                "Enter an amount FT to transfer (example: 10{symbol} or 0.5{symbol}):"
            ))
            .prompt()
            {
                Ok(mut fungible_token) => {
                    if fungible_token.symbol.to_uppercase() != symbol.to_uppercase() {
                        eprintln!("{}", "Invalid currency symbol".red());
                        continue;
                    } else if fungible_token.decimals > decimals as u8 {
                        eprintln!(
                            "{} {} {}",
                            "Invalid decimal places. Your FT amount exceeds".red(),
                            decimals.red(),
                            "decimal places.".red()
                        );
                        continue;
                    } else {
                        fungible_token.amount = fungible_token
                            .amount
                            .checked_mul(
                                10u128.pow(decimals as u32 - fungible_token.decimals as u32),
                            )
                            .wrap_err("FungibleToken: underflow or overflow happens")?;
                        fungible_token.decimals = decimals as u8;
                        fungible_token.symbol = symbol;
                        return Ok(Some(fungible_token));
                    }
                }
                Err(err) => return Err(color_eyre::Report::msg(err)),
            }
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = AmountFtContext)]
#[interactive_clap(output_context = PrepaidGasContext)]
pub struct PrepaidGas {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter gas for function call:
    gas: crate::common::NearGas,
    #[interactive_clap(named_arg)]
    /// Enter deposit for a function call
    attached_deposit: Deposit,
}

#[derive(Debug, Clone)]
pub struct PrepaidGasContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    ft_contract_account_id: near_primitives::types::AccountId,
    receiver_account_id: near_primitives::types::AccountId,
    amount_ft: crate::types::ft_properties::FungibleToken,
    gas: crate::common::NearGas,
}

impl PrepaidGasContext {
    pub fn from_previous_context(
        previous_context: AmountFtContext,
        scope: &<PrepaidGas as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            ft_contract_account_id: previous_context.ft_contract_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            amount_ft: previous_context.amount_ft,
            gas: scope.gas,
        })
    }
}

impl PrepaidGas {
    fn input_gas(
        _context: &AmountFtContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearGas>> {
        eprintln!();
        let gas = loop {
            match crate::common::NearGas::from_str(
                &Text::new("Enter gas for function call:")
                    .with_initial_value("100 TeraGas")
                    .prompt()?,
            ) {
                Ok(input_gas) => {
                    if input_gas <= near_gas::NearGas::from_tgas(300) {
                        break input_gas;
                    } else {
                        eprintln!("You need to enter a value of no more than 300 TERAGAS")
                    }
                }
                Err(err) => return Err(color_eyre::Report::msg(err)),
            }
        };
        Ok(Some(gas))
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = PrepaidGasContext)]
#[interactive_clap(output_context = DepositContext)]
pub struct Deposit {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter deposit for a function call:
    deposit: crate::types::near_token::NearToken,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct DepositContext(crate::commands::ActionContext);

impl DepositContext {
    pub fn from_previous_context(
        previous_context: PrepaidGasContext,
        scope: &<Deposit as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let signer_account_id = previous_context.signer_account_id.clone();
                let ft_contract_account_id = previous_context.ft_contract_account_id.clone();
                let receiver_account_id = previous_context.receiver_account_id.clone();
                let deposit = scope.deposit;
                let amount_ft = previous_context.amount_ft.clone();
                let action_ft_transfer = near_primitives::transaction::Action::FunctionCall(
                    near_primitives::transaction::FunctionCallAction {
                        method_name: "ft_transfer".to_string(),
                        args: serde_json::to_vec(&json!({
                            "receiver_id": receiver_account_id.to_string(),
                            "amount": amount_ft.amount.to_string()
                        }))?,
                        gas: previous_context.gas.as_gas(),
                        deposit: deposit.as_yoctonear(),
                    },
                );

                move |network_config| {
                    let args = serde_json::to_vec(&json!({
                    "account_id": receiver_account_id.to_string(),
                    }))?;
                    let call_result = network_config
                        .json_rpc_client()
                        .blocking_call_view_function(
                            &ft_contract_account_id,
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
                        let action_storage_deposit =
                            near_primitives::transaction::Action::FunctionCall(
                                near_primitives::transaction::FunctionCallAction {
                                    method_name: "storage_deposit".to_string(),
                                    args,
                                    gas: previous_context.gas.as_gas(),
                                    deposit: near_token::NearToken::from_millinear(100)
                                        .as_yoctonear(),
                                },
                            );
                        return Ok(crate::commands::PrepopulatedTransaction {
                            signer_id: signer_account_id.clone(),
                            receiver_id: ft_contract_account_id.clone(),
                            actions: vec![action_storage_deposit, action_ft_transfer.clone()],
                        });
                    }

                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: signer_account_id.clone(),
                        receiver_id: ft_contract_account_id.clone(),
                        actions: vec![action_ft_transfer.clone()],
                    })
                }
            });

        let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new({
            let signer_account_id = previous_context.signer_account_id.clone();
            let amount_ft = previous_context.amount_ft.clone();
            let ft_contract_account_id = previous_context.ft_contract_account_id.clone();
            let receiver_account_id = previous_context.receiver_account_id.clone();

            move |outcome_view, _network_config| {
                if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = outcome_view.status {
                    eprintln!(
                        "<{signer_account_id}> has successfully transferred {amount_ft} (FT-contract: {ft_contract_account_id}) to <{receiver_account_id}>.",
                    );
                }
                Ok(())
            }
        });

        Ok(Self(crate::commands::ActionContext {
            global_context: previous_context.global_context,
            interacting_with_account_ids: vec![
                previous_context.ft_contract_account_id,
                previous_context.signer_account_id,
                previous_context.receiver_account_id,
            ],
            on_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
            on_after_sending_transaction_callback,
        }))
    }
}

impl From<DepositContext> for crate::commands::ActionContext {
    fn from(item: DepositContext) -> Self {
        item.0
    }
}

impl Deposit {
    fn input_deposit(
        _context: &PrepaidGasContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::near_token::NearToken>> {
        eprintln!();
        match crate::types::near_token::NearToken::from_str(
            &Text::new(
                "Enter deposit for a function call (example: 10NEAR or 0.5near or 10000yoctonear):",
            )
            .with_initial_value("1 yoctoNEAR")
            .prompt()?,
        ) {
            Ok(deposit) => Ok(Some(deposit)),
            Err(err) => Err(color_eyre::Report::msg(err)),
        }
    }
}
