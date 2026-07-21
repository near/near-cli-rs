#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::super::ConstructTransactionContext)]
#[interactive_clap(output_context = DeleteAccountActionContext)]
pub struct DeleteAccountAction {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    /// Enter the beneficiary ID to delete this account ID:
    beneficiary_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    next_action: super::super::super::add_action_last::NextAction,
}

#[derive(Debug, Clone)]
pub struct DeleteAccountActionContext(super::super::super::ConstructTransactionContext);

impl DeleteAccountActionContext {
    pub fn from_previous_context(
        previous_context: super::super::super::ConstructTransactionContext,
        scope: &<DeleteAccountAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let beneficiary_id: near_kit::AccountId = scope.beneficiary_id.clone().into();
        if previous_context.signer_account_id == beneficiary_id {
            return Err(color_eyre::eyre::eyre!(
                "Invalid beneficiary account ID.\nThe beneficiary account ID cannot be the same as the account ID being deleted."
            ));
        }
        let action =
            near_kit::Action::DeleteAccount(near_kit::DeleteAccountAction { beneficiary_id });
        let mut actions = previous_context.actions;
        actions.push(action);
        Ok(Self(super::super::super::ConstructTransactionContext {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            actions,
            sign_as_delegate_action: previous_context.sign_as_delegate_action,
        }))
    }
}

impl From<DeleteAccountActionContext> for super::super::super::ConstructTransactionContext {
    fn from(item: DeleteAccountActionContext) -> Self {
        item.0
    }
}

impl DeleteAccountAction {
    pub fn input_beneficiary_id(
        context: &super::super::super::ConstructTransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::commands::account::delete_account::BeneficiaryAccount::input_beneficiary_account_id(
            &context.clone().into(),
        )
    }
}
