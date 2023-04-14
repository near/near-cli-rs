use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::AccountStorageManagementContext)]
#[interactive_clap(output_context = WithdrawArgsContext)]
pub struct WithdrawArgs {
    /// Under which contract account ID do you want to withdraw the deposit?
    contract_account_id: crate::types::account_id::AccountId,
    /// Enter the amount to withdraw from the storage
    amount: crate::common::NearBalance,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct WithdrawArgsContext(crate::commands::ActionContext);

impl WithdrawArgsContext {
    pub fn from_previous_context(
        previous_context: super::AccountStorageManagementContext,
        scope: &<WithdrawArgs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let args = serde_json::json!({
            "amount": scope.amount.to_yoctonear().to_string()
        })
        .to_string()
        .into_bytes();

        let account_id = previous_context.account_id.clone();
        let amount = scope.amount.clone();
        let contract: near_primitives::types::AccountId = scope.contract_account_id.clone().into();

        let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new(
            move |outcome_view, _network_config| {
                if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = outcome_view.status {
                    eprintln!(
                        "<{account_id}> has successfully withdraw {amount} from <{contract}>.",
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
                    method_name: "storage_withdraw".to_string(),
                    args,
                    gas: crate::common::NearGas::from_str("300 TeraGas")
                        .unwrap()
                        .inner,
                    deposit: crate::common::NearBalance::from_str("1 yoctoNear")
                        .unwrap()
                        .to_yoctonear(),
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

impl From<WithdrawArgsContext> for crate::commands::ActionContext {
    fn from(item: WithdrawArgsContext) -> Self {
        item.0
    }
}
