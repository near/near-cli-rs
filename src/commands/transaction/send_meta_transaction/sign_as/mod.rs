use inquire::{CustomType, Select};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::SendMetaTransactionContext)]
#[interactive_clap(output_context = RelayerAccountIdContext)]
pub struct RelayerAccountId {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the signer account ID?
    relayer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: super::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct RelayerAccountIdContext {
    pub config: crate::config::Config,
    pub signed_delegate_action: near_primitives::delegate_action::SignedDelegateAction,
    pub relayer_account_id: near_primitives::types::AccountId,
}

impl RelayerAccountIdContext {
    pub fn from_previous_context(
        previous_context: super::SendMetaTransactionContext,
        scope: &<RelayerAccountId as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            signed_delegate_action: previous_context.signed_delegate_action,
            relayer_account_id: scope.relayer_account_id.clone().into(),
        })
    }
}

impl RelayerAccountId {
    fn input_relayer_account_id(
        context: &super::SendMetaTransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        loop {
            let relayer_account_id: crate::types::account_id::AccountId =
                CustomType::new(" What is the relayer account ID?").prompt()?;
            if !crate::common::is_account_exist(
                &context.config.network_connection,
                relayer_account_id.clone().into(),
            ) {
                eprintln!("\nThe account <{relayer_account_id}> does not yet exist.");
                #[derive(strum_macros::Display)]
                enum ConfirmOptions {
                    #[strum(to_string = "Yes, I want to enter a new account name.")]
                    Yes,
                    #[strum(to_string = "No, I want to use this account name.")]
                    No,
                }
                let select_choose_input = Select::new(
                    "Do you want to enter another relayer account id?",
                    vec![ConfirmOptions::Yes, ConfirmOptions::No],
                )
                .prompt()?;
                if let ConfirmOptions::No = select_choose_input {
                    return Ok(Some(relayer_account_id));
                }
            } else {
                return Ok(Some(relayer_account_id));
            }
        }
    }
}
