use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::ContractContext)]
#[interactive_clap(output_context = WithdrawArgsContext)]
pub struct WithdrawArgs {
    /// Enter the amount to withdraw from the storage (example: 10NEAR or 0.5near or 10000yoctonear)
    amount: crate::common::NearBalance,
    #[interactive_clap(named_arg)]
    /// What is the signer account ID?
    sign_as: SignerAccountId,
}

#[derive(Clone)]
pub struct WithdrawArgsContext {
    config: crate::config::Config,
    get_contract_account_id: super::GetContractAccountID,
    amount: crate::common::NearBalance,
}

impl WithdrawArgsContext {
    pub fn from_previous_context(
        previous_context: super::ContractContext,
        scope: &<WithdrawArgs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            get_contract_account_id: previous_context.get_contract_account_id,
            amount: scope.amount.clone(),
        })
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = WithdrawArgsContext)]
#[interactive_clap(output_context = SignerAccountIdContext)]
pub struct SignerAccountId {
    /// What is the signer account ID?
    signer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct SignerAccountIdContext(crate::commands::ActionContext);

impl SignerAccountIdContext {
    pub fn from_previous_context(
        previous_context: WithdrawArgsContext,
        scope: &<SignerAccountId as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let account_id = scope.signer_account_id.clone();
        let amount = previous_context.amount.clone();
        let get_contract_account_id = previous_context.get_contract_account_id.clone();

        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::sync::Arc::new(move |prepolulated_unsinged_transaction, network_config| {
                let contract_account_id = (get_contract_account_id)(network_config)?;
                prepolulated_unsinged_transaction.receiver_id = contract_account_id;
                Ok(())
            });

        let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new(
            move |outcome_view, network_config| {
                if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = outcome_view.status {
                    let contract_account_id = (previous_context.get_contract_account_id)(network_config)?;

                    eprintln!(
                        "<{account_id}> has successfully withdraw {amount} from <{contract_account_id}>.",
                    );
                }
                Ok(())
            },
        );

        Ok(Self(crate::commands::ActionContext {
            config: previous_context.config,
            signer_account_id: scope.signer_account_id.clone().into(),
            receiver_account_id: scope.signer_account_id.clone().into(), //contract_account_id,
            actions: vec![near_primitives::transaction::Action::FunctionCall(
                near_primitives::transaction::FunctionCallAction {
                    method_name: "storage_withdraw".to_string(),
                    args: serde_json::json!({
                        "amount": previous_context.amount.to_yoctonear().to_string()
                    })
                    .to_string()
                    .into_bytes(),
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
            on_after_getting_network_callback,
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
            on_after_sending_transaction_callback,
        }))
    }
}

impl From<SignerAccountIdContext> for crate::commands::ActionContext {
    fn from(item: SignerAccountIdContext) -> Self {
        item.0
    }
}
