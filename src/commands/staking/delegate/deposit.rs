#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::DelegateStakeContext)]
#[interactive_clap(output_context = DepositContext)]
pub struct Deposit {
    /// Enter the attached amount to be deposited into the predecessor's internal account (example: 10NEAR or 0.5near or 10000yoctonear):
    amount: crate::common::NearBalance,
    #[interactive_clap(skip_default_input_arg)]
    /// What is the signer account ID?
    signer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct DepositContext(crate::commands::ActionContext);

impl DepositContext {
    pub fn from_previous_context(
        previous_context: super::DelegateStakeContext,
        scope: &<Deposit as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let signer = scope.signer_account_id.clone();
        let signer_id: near_primitives::types::AccountId = scope.signer_account_id.clone().into();
        let validator_account_id = previous_context.validator_account_id.clone();
        let interacting_with_account_ids = vec![validator_account_id.clone()];
        let amount = scope.amount.clone();

        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::sync::Arc::new(move |_network_config| {
                Ok(crate::commands::PrepopulatedTransaction {
                    signer_id: signer_id.clone(),
                    receiver_id: previous_context.validator_account_id.clone(),
                    actions: vec![near_primitives::transaction::Action::FunctionCall(
                        near_primitives::transaction::FunctionCallAction {
                            method_name: "deposit".to_string(),
                            args: serde_json::json!({}).to_string().into_bytes(),
                            gas: crate::common::NearGas::from_tgas(300).as_gas(),
                            deposit: amount.to_yoctonear(),
                        },
                    )],
                })
            });

        let amount = scope.amount.clone();
        let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new(
                move |outcome_view, _network_config| {
                    if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = outcome_view.status {
                        eprintln!("<{signer}> has successfully deposited {amount} on <{validator_account_id}>.")
                    }
                    Ok(())
                },
            );
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

impl From<DepositContext> for crate::commands::ActionContext {
    fn from(item: DepositContext) -> Self {
        item.0
    }
}

impl Deposit {
    pub fn input_signer_account_id(
        context: &super::DelegateStakeContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the signer account ID?",
        )
    }
}
