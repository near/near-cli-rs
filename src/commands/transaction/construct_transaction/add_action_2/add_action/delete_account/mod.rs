use color_eyre::owo_colors::OwoColorize;
use inquire::Select;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::super::ConstructTransactionContext)]
#[interactive_clap(output_context = DeleteAccountActionContext)]
pub struct DeleteAccountAction {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    /// Enter the beneficiary ID to delete this account ID:
    beneficiary_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    next_action: super::super::super::add_action_3::NextAction,
}

#[derive(Debug, Clone)]
pub struct DeleteAccountActionContext(super::super::super::ConstructTransactionContext);

impl DeleteAccountActionContext {
    pub fn from_previous_context(
        previous_context: super::super::super::ConstructTransactionContext,
        scope: &<DeleteAccountAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let beneficiary_id: near_primitives::types::AccountId = scope.beneficiary_id.clone().into();
        let action = near_primitives::transaction::Action::DeleteAccount(
            near_primitives::transaction::DeleteAccountAction { beneficiary_id },
        );
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

impl From<DeleteAccountActionContext> for super::super::super::ConstructTransactionContext {
    fn from(item: DeleteAccountActionContext) -> Self {
        item.0
    }
}

impl DeleteAccountAction {
    pub fn input_beneficiary_id(
        context: &super::super::super::ConstructTransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        loop {
            let beneficiary_account_id = if let Some(account_id) =
                crate::common::input_non_signer_account_id_from_used_account_list(
                    &context.global_context.config.credentials_home_dir,
                    "What is the beneficiary account ID?",
                )? {
                account_id
            } else {
                return Ok(None);
            };

            if beneficiary_account_id.0 == context.signer_account_id {
                eprintln!("{}", "You have selected a beneficiary account ID that will now be deleted. This will result in the loss of your funds. So make your choice again.".red());
                continue;
            }

            if context.global_context.offline {
                return Ok(Some(beneficiary_account_id));
            }

            #[derive(derive_more::Display)]
            enum ConfirmOptions {
                #[display("Yes, I want to check if account <{account_id}> exists. (It is free of charge, and only requires Internet access)")]
                Yes {
                    account_id: crate::types::account_id::AccountId,
                },
                #[display("No, I know this account exists and want to continue.")]
                No,
            }
            let select_choose_input =
                Select::new("\nDo you want to check the existence of the specified account so that you don't lose tokens?",
                    vec![ConfirmOptions::Yes{account_id: beneficiary_account_id.clone()}, ConfirmOptions::No],
                    )
                    .prompt()?;
            if let ConfirmOptions::Yes { account_id } = select_choose_input {
                if crate::common::find_network_where_account_exist(
                    &context.global_context,
                    account_id.clone().into(),
                )?
                .is_none()
                {
                    eprintln!(
                        "\nHeads up! You will lose remaining NEAR tokens on the account you delete if you specify the account <{}> as the beneficiary as it does not exist on [{}] networks.",
                        account_id,
                        context.global_context.config.network_names().join(", ")
                    );
                    if !crate::common::ask_if_different_account_id_wanted()? {
                        return Ok(Some(account_id));
                    }
                } else {
                    return Ok(Some(account_id));
                };
            } else {
                return Ok(Some(beneficiary_account_id));
            };
        }
    }
}
