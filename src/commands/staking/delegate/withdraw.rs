#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::StakeDelegationContext)]
#[interactive_clap(output_context = WithdrawContext)]
pub struct Withdraw {
    /// Enter the amount to withdraw from the non staked balance (example: 10NEAR or 0.5near or 10000yoctonear):
    amount: near_token::NearToken,
    #[interactive_clap(skip_default_input_arg)]
    /// What is validator account ID?
    validator_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct WithdrawContext(crate::commands::ActionContext);

impl WithdrawContext {
    pub fn from_previous_context(
        previous_context: super::StakeDelegationContext,
        scope: &<Withdraw as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::rc::Rc::new({
                let signer_id = previous_context.account_id.clone();
                let validator_account_id: near_primitives::types::AccountId =
                    scope.validator_account_id.clone().into();
                let amount = scope.amount;

                move |_network_config| {
                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: signer_id.clone(),
                        receiver_id: validator_account_id.clone(),
                        actions: vec![near_primitives::transaction::Action::FunctionCall(
                            near_primitives::transaction::FunctionCallAction {
                                method_name: "withdraw".to_string(),
                                args: serde_json::to_vec(&serde_json::json!({
                                    "amount": amount,
                                }))?,
                                gas: crate::common::NearGas::from_tgas(50).as_gas(),
                                deposit: 0,
                            },
                        )],
                    })
                }
            });

        let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = std::rc::Rc::new({
            let signer_id = previous_context.account_id.clone();
            let validator_id = scope.validator_account_id.clone();
            let amount = scope.amount;

            move |outcome_view, _network_config| {
                if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = outcome_view.status {
                    eprintln!("<{signer_id}> has successfully withdrawn {amount} from <{validator_id}>.")
                }
                Ok(())
            }
        });

        Ok(Self(crate::commands::ActionContext {
            global_context: previous_context.global_context,
            interacting_with_account_ids: vec![
                previous_context.account_id,
                scope.validator_account_id.clone().into(),
            ],
            on_after_getting_network_callback,
            on_before_signing_callback: std::rc::Rc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::rc::Rc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
            on_after_sending_transaction_callback,
        }))
    }
}

impl From<WithdrawContext> for crate::commands::ActionContext {
    fn from(item: WithdrawContext) -> Self {
        item.0
    }
}

impl Withdraw {
    pub fn input_validator_account_id(
        context: &super::StakeDelegationContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_staking_pool_validator_account_id(&context.global_context.config)
    }
}
