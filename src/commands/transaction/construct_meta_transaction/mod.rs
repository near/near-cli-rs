#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ConstructMetaTransactionContext)]
pub struct ConstructMetaTransaction {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the sender account ID?
    pub sender_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(skip_default_input_arg)]
    /// What is the receiver account ID?
    pub receiver_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    pub next_actions: crate::commands::transaction::construct_transaction::add_action_1::NextAction,
}

#[derive(Debug, Clone)]
pub struct ConstructMetaTransactionContext(
    crate::commands::transaction::construct_transaction::ConstructTransactionContext,
);

impl ConstructMetaTransactionContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<ConstructMetaTransaction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(
            crate::commands::transaction::construct_transaction::ConstructTransactionContext {
                global_context: previous_context,
                signer_account_id: scope.sender_account_id.clone().into(),
                receiver_account_id: scope.receiver_account_id.clone().into(),
                actions: vec![],
                sign_as_delegate_action: true,
            },
        ))
    }
}

impl From<ConstructMetaTransactionContext>
    for crate::commands::transaction::construct_transaction::ConstructTransactionContext
{
    fn from(item: ConstructMetaTransactionContext) -> Self {
        item.0
    }
}

impl ConstructMetaTransaction {
    pub fn input_sender_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is the sender account ID?",
        )
    }

    pub fn input_receiver_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is the receiver account ID?",
        )
    }
}
