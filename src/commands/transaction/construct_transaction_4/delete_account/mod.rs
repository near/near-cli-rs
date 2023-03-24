use async_recursion::async_recursion;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct DeleteAccountAction {
    #[interactive_clap(long)]
    ///Enter the beneficiary ID to delete this account ID
    beneficiary_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    next_action: super::BoxNextAction,
}

impl DeleteAccountAction {
    #[async_recursion(?Send)]
    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        let beneficiary_id: near_primitives::types::AccountId = self.beneficiary_id.clone().into();
        let action = near_primitives::transaction::Action::DeleteAccount(
            near_primitives::transaction::DeleteAccountAction { beneficiary_id },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        match *self.next_action.clone().inner {
            super::NextAction::AddAction(select_action) => {
                select_action.process(config, unsigned_transaction).await
            }
            super::NextAction::Skip(skip_action) => {
                skip_action.process(config, unsigned_transaction).await
            }
        }
    }
}
