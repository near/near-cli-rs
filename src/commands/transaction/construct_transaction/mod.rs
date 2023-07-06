mod add_action_1;
mod add_action_2;
mod add_action_3;
mod add_action_last;
mod skip_action;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ConstructTransactionContext)]
pub struct ConstructTransaction {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the sender account ID?
    sender_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(skip_default_input_arg)]
    /// What is the receiver account ID?
    receiver_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    next_actions: self::add_action_1::NextAction,
}

#[derive(Debug, Clone)]
pub struct ConstructTransactionContext {
    pub global_context: crate::GlobalContext,
    pub signer_account_id: near_primitives::types::AccountId,
    pub receiver_account_id: near_primitives::types::AccountId,
    pub actions: Vec<near_primitives::transaction::Action>,
}

impl ConstructTransactionContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<ConstructTransaction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            signer_account_id: scope.sender_account_id.clone().into(),
            receiver_account_id: scope.receiver_account_id.clone().into(),
            actions: vec![],
        })
    }
}

impl ConstructTransaction {
    pub fn input_sender_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        Ok(Some(
            crate::common::input_account_id_from_used_account_list(
                &context.config.credentials_home_dir,
                "What is the sender account ID?",
                true,
            )?,
        ))
    }

    pub fn input_receiver_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        Ok(Some(
            crate::common::input_account_id_from_used_account_list(
                &context.config.credentials_home_dir,
                "What is the receiver account ID?",
                false,
            )?,
        ))
    }
}
