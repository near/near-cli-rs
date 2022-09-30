use async_recursion::async_recursion;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct CreateSubAccountAction {
    #[interactive_clap(subcommand)]
    next_action: super::BoxNextAction,
}

impl CreateSubAccountAction {
    #[async_recursion(?Send)]
    pub async fn process(
        &self,
        config: crate::config::Config,
        mut prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        let action = near_primitives::transaction::Action::CreateAccount(
            near_primitives::transaction::CreateAccountAction {},
        );
        prepopulated_unsigned_transaction.actions.push(action);
        match *self.next_action.clone().inner {
            super::NextAction::AddAction(select_action) => {
                select_action
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
            super::NextAction::Skip(skip_action) => {
                skip_action
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
        }
    }
}
