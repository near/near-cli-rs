use std::str::FromStr;

use inquire::{CustomType, Select};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::AccountStorageManagementContext)]
#[interactive_clap(output_context = DepositArgsContext)]
pub struct DepositArgs {
    /// Under which contract account ID do you want to withdraw the deposit?
    contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(skip_default_input_arg)]
    /// Which account ID do you want to add a deposit to?
    receiver_account_id: crate::types::account_id::AccountId,
    /// Enter the amount to deposit into the storage (example: 10NEAR or 0.5near or 10000yoctonear)
    deposit: crate::common::NearBalance,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct DepositArgsContext(crate::commands::ActionContext);

impl DepositArgsContext {
    pub fn from_previous_context(
        previous_context: super::AccountStorageManagementContext,
        scope: &<DepositArgs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let account_id = previous_context.account_id.clone();
        let amount = scope.deposit.clone();
        let contract = scope.contract_account_id.clone();
        let receiver = scope.receiver_account_id.clone();

        let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new(
            move |outcome_view, _network_config| {
                if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = outcome_view.status {
                    eprintln!(
                        "<{account_id}> has successfully added a deposit of {amount} to <{receiver}> on contract <{contract}>.",
                    );
                }
                Ok(())
            },
        );

        Ok(Self(crate::commands::ActionContext {
            config: previous_context.config,
            signer_account_id: previous_context.account_id.clone(),
            receiver_account_id: scope.contract_account_id.clone().into(),
            actions: vec![near_primitives::transaction::Action::FunctionCall(
                near_primitives::transaction::FunctionCallAction {
                    method_name: "storage_deposit".to_string(),
                    args: serde_json::json!({
                        "account_id": scope.receiver_account_id.to_string()
                    })
                    .to_string()
                    .into_bytes(),
                    gas: crate::common::NearGas::from_str("300 TeraGas")
                        .unwrap()
                        .inner,
                    deposit: scope.deposit.clone().to_yoctonear(),
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
        }))
    }
}

impl From<DepositArgsContext> for crate::commands::ActionContext {
    fn from(item: DepositArgsContext) -> Self {
        item.0
    }
}

impl DepositArgs {
    fn input_receiver_account_id(
        context: &super::AccountStorageManagementContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        loop {
            let receiver_account_id: crate::types::account_id::AccountId =
                CustomType::new(" What is the receiver account ID?")
                    .with_default(context.account_id.clone().into())
                    .prompt()?;
            if !crate::common::is_account_exist(
                &context.config.network_connection,
                receiver_account_id.clone().into(),
            ) {
                eprintln!("\nThe account <{receiver_account_id}> does not yet exist.");
                #[derive(strum_macros::Display)]
                enum ConfirmOptions {
                    #[strum(to_string = "Yes, I want to enter a new account name.")]
                    Yes,
                    #[strum(to_string = "No, I want to use this account name.")]
                    No,
                }
                let select_choose_input = Select::new(
                    "Do you want to enter another receiver account id?",
                    vec![ConfirmOptions::Yes, ConfirmOptions::No],
                )
                .prompt()?;
                if let ConfirmOptions::No = select_choose_input {
                    return Ok(Some(receiver_account_id));
                }
            } else {
                return Ok(Some(receiver_account_id));
            }
        }
    }
}
