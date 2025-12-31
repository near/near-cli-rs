#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::StakeDelegationContext)]
#[interactive_clap(output_context = StakeContext)]
pub struct Stake {
    /// Enter the amount to stake from the inner account of the predecessor (example: 10 NEAR or 0.5 NEAR or 10000 yoctonear):
    amount: crate::types::near_token::NearToken,
    #[interactive_clap(skip_default_input_arg)]
    /// What is validator account ID?
    validator_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct StakeContext(crate::commands::ActionContext);

impl StakeContext {
    pub fn from_previous_context(
        previous_context: super::StakeDelegationContext,
        scope: &<Stake as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let signer_id = previous_context.account_id.clone();
                let validator_account_id: near_primitives::types::AccountId =
                    scope.validator_account_id.clone().into();
                let amount = scope.amount;

                move |_network_config| {
                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: signer_id.clone(),
                        receiver_id: validator_account_id.clone(),
                        actions: vec![near_primitives::transaction::Action::FunctionCall(
                            Box::new(near_primitives::transaction::FunctionCallAction {
                                method_name: "stake".to_string(),
                                args: serde_json::to_vec(&serde_json::json!({
                                    "amount": amount,
                                }))?,
                                gas: near_primitives::gas::Gas::from_teragas(50),
                                deposit: near_token::NearToken::ZERO,
                            }),
                        )],
                    })
                }
            });

        let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new({
            let signer_id = previous_context.account_id.clone();
            let validator_id = scope.validator_account_id.clone();
            let amount = scope.amount;
            let verbosity = previous_context.global_context.verbosity;

            move |outcome_view, _network_config| {
                if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = outcome_view.status {
                    if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe = verbosity {
                        tracing_indicatif::suspend_tracing_indicatif(|| {
                            eprintln!("<{signer_id}> has successfully delegated {amount} to stake with <{validator_id}>.");
                        })
                    }
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

impl From<StakeContext> for crate::commands::ActionContext {
    fn from(item: StakeContext) -> Self {
        item.0
    }
}

impl Stake {
    pub fn input_validator_account_id(
        context: &super::StakeDelegationContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_staking_pool_validator_account_id(&context.global_context.config)
    }
}
