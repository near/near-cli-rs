pub mod add_action_1;
pub mod add_action_2;
pub mod add_action_3;
pub mod add_action_last;
pub mod skip_action;
pub mod state_init_receiver;

use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ConstructTransactionSenderContext)]
pub struct ConstructTransaction {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the sender account ID?
    pub sender_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    pub receiver: ReceiverMode,
}

#[derive(Debug, Clone)]
pub struct ConstructTransactionSenderContext {
    pub global_context: crate::GlobalContext,
    pub signer_account_id: near_primitives::types::AccountId,
}

impl ConstructTransactionSenderContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<ConstructTransaction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            signer_account_id: scope.sender_account_id.clone().into(),
        })
    }
}

impl ConstructTransaction {
    pub fn input_sender_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is the sender account ID?",
        )
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = ConstructTransactionSenderContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How would you like to specify the receiver?
pub enum ReceiverMode {
    #[strum_discriminants(strum(message = "receiver-id  - Specify receiver account ID directly"))]
    ReceiverId(DirectReceiver),
    #[strum_discriminants(strum(
        message = "state-init  - Derive receiver from deterministic state init (NEP-616)"
    ))]
    StateInit(self::state_init_receiver::StateInitReceiver),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = ConstructTransactionSenderContext)]
#[interactive_clap(output_context = ConstructTransactionContext)]
pub struct DirectReceiver {
    #[interactive_clap(skip_default_input_arg)]
    pub receiver_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    pub next_actions: self::add_action_1::NextAction,
}

impl DirectReceiver {
    pub fn input_receiver_account_id(
        context: &ConstructTransactionSenderContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the receiver account ID?",
        )
    }
}

#[derive(Debug, Clone)]
pub struct ConstructTransactionContext {
    pub global_context: crate::GlobalContext,
    pub signer_account_id: near_primitives::types::AccountId,
    pub receiver_account_id: near_primitives::types::AccountId,
    pub actions: Vec<near_primitives::transaction::Action>,
    pub sign_as_delegate_action: bool,
}

impl ConstructTransactionContext {
    pub fn from_previous_context(
        previous_context: ConstructTransactionSenderContext,
        scope: &<DirectReceiver as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            receiver_account_id: scope.receiver_account_id.clone().into(),
            actions: vec![],
            sign_as_delegate_action: false,
        })
    }
}
