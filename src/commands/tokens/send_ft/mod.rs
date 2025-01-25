use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod exact_amount_ft;
mod max_amount_ft;
mod transaction_formation;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::TokensCommandsContext)]
#[interactive_clap(output_context = SendFtCommandContext)]
pub struct SendFtCommand {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the ft-contract account ID?
    ft_contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(skip_default_input_arg)]
    /// What is the receiver account ID?
    receiver_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    transfer_amount_ft: TransferAmountFt,
}

#[derive(Debug, Clone)]
pub struct SendFtCommandContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    ft_contract_account_id: near_primitives::types::AccountId,
    receiver_account_id: near_primitives::types::AccountId,
}

impl SendFtCommandContext {
    pub fn from_previous_context(
        previous_context: super::TokensCommandsContext,
        scope: &<SendFtCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.owner_account_id,
            ft_contract_account_id: scope.ft_contract_account_id.clone().into(),
            receiver_account_id: scope.receiver_account_id.clone().into(),
        })
    }
}

impl SendFtCommand {
    pub fn input_ft_contract_account_id(
        context: &super::TokensCommandsContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the ft-contract account ID?",
        )
    }

    pub fn input_receiver_account_id(
        context: &super::TokensCommandsContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the receiver account ID?",
        )
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = SendFtCommandContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select an action with the amount of fungible tokens to transfer:
pub enum TransferAmountFt {
    #[strum_discriminants(strum(
        message = "exact-amount   - Transfer of the specified amount of fungible tokens (wNearAmount (10 wNEAR))"
    ))]
    /// Transfer of the specified amount of fungible tokens (wNearAmount (10 wNEAR))
    ExactAmount(self::exact_amount_ft::ExactAmountFt),
    #[strum_discriminants(strum(
        message = "max-amount     - Transfer the entire amount of fungible tokens from your account ID"
    ))]
    /// Transfer the entire amount of fungible tokens from your account ID
    MaxAmount(self::max_amount_ft::MaxAmountFt),
}
