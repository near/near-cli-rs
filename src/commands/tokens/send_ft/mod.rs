use std::str::FromStr;

use inquire::Text;
use serde_json::json;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::TokensCommandsContext)]
#[interactive_clap(output_context = crate::commands::ActionContext)]
pub struct SendFtCommand {
    /// What is the ft-contract account ID?
    ft_contract_account_id: crate::types::account_id::AccountId,
    /// What is the receiver account ID?
    receiver_account_id: crate::types::account_id::AccountId,
    /// Enter an amount FT to transfer
    amount: u128,
    #[interactive_clap(long = "prepaid-gas")]
    #[interactive_clap(skip_default_input_arg)]
    /// Enter gas for function call
    gas: crate::common::NearGas,
    #[interactive_clap(long = "attached-deposit")]
    #[interactive_clap(skip_default_input_arg)]
    /// Enter deposit for a function call
    deposit: crate::common::NearBalance,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct SendFtCommandContext {
    config: crate::config::Config,
    signer_account_id: near_primitives::types::AccountId,
    ft_contract_account_id: near_primitives::types::AccountId,
    receiver_account_id: near_primitives::types::AccountId,
    amount: u128,
    gas: crate::common::NearGas,
    deposit: crate::common::NearBalance,
}

impl SendFtCommandContext {
    pub fn from_previous_context(
        previous_context: super::TokensCommandsContext,
        scope: &<SendFtCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            signer_account_id: previous_context.owner_account_id.into(),
            ft_contract_account_id: scope.ft_contract_account_id.clone().into(),
            receiver_account_id: scope.receiver_account_id.clone().into(),
            amount: scope.amount,
            gas: scope.gas.clone(),
            deposit: scope.deposit.clone(),
        })
    }
}

impl From<SendFtCommandContext> for crate::commands::ActionContext {
    fn from(item: SendFtCommandContext) -> Self {
        let method_name = "ft_transfer".to_string();
        let args = json!({
            "receiver_id": item.receiver_account_id.to_string(),
            "amount": item.amount.to_string()
        })
        .to_string()
        .into_bytes();
        let sender = item.signer_account_id.clone();
        let amount = item.amount.clone();
        let contract = item.ft_contract_account_id.clone();
        let receiver = item.receiver_account_id.clone();

        let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new(
            move |outcome_view, _network_config| {
                if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = outcome_view.status {
                    println!(
                        "<{sender}> has successfully transferred {amount} FT ({contract}) to <{receiver}>.",
                    );
                }
                Ok(())
            },
        );

        Self {
            config: item.config,
            signer_account_id: item.signer_account_id,
            receiver_account_id: item.ft_contract_account_id,
            actions: vec![near_primitives::transaction::Action::FunctionCall(
                near_primitives::transaction::FunctionCallAction {
                    method_name,
                    args,
                    gas: item.gas.clone().inner,
                    deposit: item.deposit.clone().to_yoctonear(),
                },
            )],
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_after_getting_network_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
            on_after_sending_transaction_callback,
        }
    }
}

impl SendFtCommand {
    fn input_gas(
        _context: &super::TokensCommandsContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearGas>> {
        println!();
        let gas: u64 = loop {
            match crate::common::NearGas::from_str(
                &Text::new("Enter gas for function call")
                    .with_initial_value("100 TeraGas")
                    .prompt()?,
            ) {
                Ok(input_gas) => {
                    let crate::common::NearGas { inner: num } = input_gas;
                    let gas = num;
                    if gas <= 300000000000000 {
                        break gas;
                    } else {
                        println!("You need to enter a value of no more than 300 TERAGAS")
                    }
                }
                Err(err) => return Err(color_eyre::Report::msg(err)),
            }
        };
        Ok(Some(gas.into()))
    }

    fn input_deposit(
        _context: &super::TokensCommandsContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearBalance>> {
        println!();
        match crate::common::NearBalance::from_str(
            &Text::new(
                "Enter deposit for a function call (example: 10NEAR or 0.5near or 10000yoctonear).",
            )
            .with_initial_value("1 yoctoNEAR")
            .prompt()?,
        ) {
            Ok(deposit) => Ok(Some(deposit)),
            Err(err) => Err(color_eyre::Report::msg(err)),
        }
    }
}
