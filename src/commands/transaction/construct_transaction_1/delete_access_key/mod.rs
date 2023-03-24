use async_recursion::async_recursion;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct DeleteKeyCommand {
    ///Enter the public key You wish to delete
    public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(subcommand)]
    next_action: super::BoxNextAction,
}

impl DeleteKeyCommand {
    #[async_recursion(?Send)]
    pub async fn process(
        &self,
        config: crate::config::Config,
        mut prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        let action = near_primitives::transaction::Action::DeleteKey(
            near_primitives::transaction::DeleteKeyAction {
                public_key: self.public_key.clone().into(),
            },
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
