use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::super::super::ConstructTransactionContext)]
pub struct UseGlobalContractAction {
    #[interactive_clap(subcommand)]
    mode: UseGlobalActionMode,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::super::super::ConstructTransactionContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// Choose a global contract deploy mode:
pub enum UseGlobalActionMode {
    #[strum_discriminants(strum(
        message = "use-global-hash       - Use a global contract code hash pre-deployed on-chain (immutable)"
    ))]
    /// Use a global contract code hash (immutable)
    UseGlobalHash(UseHashAction),
    #[strum_discriminants(strum(
        message = "use-global-account-id - Use a global contract account ID pre-deployed on-chain (mutable)"
    ))]
    /// Use a global contract account ID (mutable)
    UseGlobalAccountId(UseAccountIdAction),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::super::ConstructTransactionContext)]
#[interactive_clap(output_context = UseHashActionContext)]
pub struct UseHashAction {
    /// What is the hash of the global contract?
    pub hash: crate::types::crypto_hash::CryptoHash,
    #[interactive_clap(subcommand)]
    initialize: super::deploy_contract::initialize_mode::InitializeMode,
}

#[derive(Debug, Clone)]
pub struct UseHashActionContext(super::super::super::ConstructTransactionContext);

impl UseHashActionContext {
    pub fn from_previous_context(
        previous_context: super::super::super::ConstructTransactionContext,
        scope: &<UseHashAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let action = near_primitives::transaction::Action::UseGlobalContract(Box::new(
            near_primitives::action::UseGlobalContractAction {
                contract_identifier: near_primitives::action::GlobalContractIdentifier::CodeHash(
                    scope.hash.into(),
                ),
            },
        ));
        let mut actions = previous_context.actions;
        actions.push(action);
        Ok(Self(super::super::super::ConstructTransactionContext {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            actions,
        }))
    }
}
impl From<UseHashActionContext> for super::super::super::ConstructTransactionContext {
    fn from(item: UseHashActionContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::super::ConstructTransactionContext)]
#[interactive_clap(output_context = UseAccountIdActionContext)]
pub struct UseAccountIdAction {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the account ID of the global contract?
    pub account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    initialize: super::deploy_contract::initialize_mode::InitializeMode,
}

impl UseAccountIdAction {
    pub fn input_account_id(
        context: &super::super::super::ConstructTransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the account ID of the global contract?",
        )
    }
}

#[derive(Debug, Clone)]
pub struct UseAccountIdActionContext(super::super::super::ConstructTransactionContext);

impl UseAccountIdActionContext {
    pub fn from_previous_context(
        previous_context: super::super::super::ConstructTransactionContext,
        scope: &<UseAccountIdAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let action = near_primitives::transaction::Action::UseGlobalContract(Box::new(
            near_primitives::action::UseGlobalContractAction {
                contract_identifier: near_primitives::action::GlobalContractIdentifier::AccountId(
                    scope.account_id.clone().into(),
                ),
            },
        ));
        let mut actions = previous_context.actions;
        actions.push(action);
        Ok(Self(super::super::super::ConstructTransactionContext {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            actions,
        }))
    }
}

impl From<UseAccountIdActionContext> for super::super::super::ConstructTransactionContext {
    fn from(item: UseAccountIdActionContext) -> Self {
        item.0
    }
}
