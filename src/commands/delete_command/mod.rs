use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod access_key;
mod account;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = ())]
pub struct DeleteAction {
    #[interactive_clap(subcommand)]
    pub action: Action,
}

impl DeleteAction {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.action.process(prepopulated_unsigned_transaction).await
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = ())]
///Сhoose what you want to delete
pub enum Action {
    #[strum_discriminants(strum(message = "Delete an access key for this account"))]
    /// Delete an access key for an account
    AccessKey(self::access_key::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "Delete this account"))]
    /// Delete this account
    Account(self::account::operation_mode::OperationMode),
}

impl Action {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self {
            Action::AccessKey(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
            Action::Account(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
        }
    }
}
