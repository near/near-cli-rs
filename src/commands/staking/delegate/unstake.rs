#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::StakeDelegationContext)]
#[interactive_clap(output_context = UnstakeContext)]
pub struct Unstake {
    /// Enter the amount to unstake from the inner account of the predecessor (example: 10NEAR or 0.5near or 10000yoctonear):
    amount: crate::common::NearBalance,
    #[interactive_clap(skip_default_input_arg)]
    /// What is validator account ID?
    validator_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct UnstakeContext(crate::commands::ActionContext);

impl UnstakeContext {
    pub fn from_previous_context(
        previous_context: super::StakeDelegationContext,
        scope: &<Unstake as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let signer = previous_context.account_id.clone();
        let validator_account_id: near_primitives::types::AccountId =
            scope.validator_account_id.clone().into();
        let validator_id = validator_account_id.clone();
        let interacting_with_account_ids = vec![
            previous_context.account_id.clone(),
            validator_account_id.clone(),
        ];
        let amount = scope.amount.clone();

        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::sync::Arc::new(move |_network_config| {
                Ok(crate::commands::PrepopulatedTransaction {
                    signer_id: previous_context.account_id.clone(),
                    receiver_id: validator_account_id.clone(),
                    actions: vec![near_primitives::transaction::Action::FunctionCall(
                        near_primitives::transaction::FunctionCallAction {
                            method_name: "unstake".to_string(),
                            args: serde_json::to_vec(&serde_json::json!({
                                "amount": amount.clone().to_yoctonear().to_string()
                            }))?,
                            gas: crate::common::NearGas::from_tgas(300).as_gas(),
                            deposit: 0,
                        },
                    )],
                })
            });

        let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new({
                let amount = scope.amount.clone();

                move |outcome_view, _network_config| {
                    if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = outcome_view.status {
                        eprintln!("<{signer}> has successfully unstake {amount} from <{validator_id}>.")
                    }
                    Ok(())
                }
            });
        Ok(Self(crate::commands::ActionContext {
            global_context: previous_context.global_context,
            interacting_with_account_ids,
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

impl From<UnstakeContext> for crate::commands::ActionContext {
    fn from(item: UnstakeContext) -> Self {
        item.0
    }
}

impl Unstake {
    pub fn input_validator_account_id(
        context: &super::StakeDelegationContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_delegated_validator_account_id_from_used_delegated_validator_list(
            &context.global_context.config.credentials_home_dir,
        )
    }
}
