#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::StakeDelegationContext)]
#[interactive_clap(output_context = UnstakeAllContext)]
pub struct UnstakeAll {
    #[interactive_clap(skip_default_input_arg)]
    /// What is validator account ID?
    validator_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct UnstakeAllContext(crate::commands::ActionContext);

impl UnstakeAllContext {
    pub fn from_previous_context(
        previous_context: super::StakeDelegationContext,
        scope: &<UnstakeAll as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let signer_id = previous_context.account_id.clone();
                let validator_account_id: near_primitives::types::AccountId =
                    scope.validator_account_id.clone().into();

                move |_network_config| {
                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: signer_id.clone(),
                        receiver_id: validator_account_id.clone(),
                        actions: vec![near_primitives::transaction::Action::FunctionCall(
                            Box::new(near_primitives::transaction::FunctionCallAction {
                                method_name: "unstake_all".to_string(),
                                args: serde_json::to_vec(&serde_json::json!({}))?,
                                gas: crate::common::NearGas::from_tgas(50).as_gas(),
                                deposit: 0,
                            }),
                        )],
                    })
                }
            });

        let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new({
            let signer_id = previous_context.account_id.clone();
            let validator_id = scope.validator_account_id.clone();

            move |outcome_view, _network_config| {
                if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = outcome_view.status {
                    tracing::info!(
                        parent: &tracing::Span::none(),
                        "\n{}",
                        crate::common::indent_payload(&format!("<{signer_id}> has successfully unstaked the entire available amount from <{validator_id}>."))
                    );
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
            get_prepopulated_transaction_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepopulated_unsigned_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
            on_after_sending_transaction_callback,
        }))
    }
}

impl From<UnstakeAllContext> for crate::commands::ActionContext {
    fn from(item: UnstakeAllContext) -> Self {
        item.0
    }
}

impl UnstakeAll {
    pub fn input_validator_account_id(
        context: &super::StakeDelegationContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_staking_pool_validator_account_id(&context.global_context.config)
    }
}
