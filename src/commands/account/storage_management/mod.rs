use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod view_storage_balance;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = AccountStorageManagementContext)]
pub struct AccountStorageManagement {
    /// What is your account ID?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    storage_actions: StorageActions,
}

#[derive(Debug, Clone)]
pub struct AccountStorageManagementContext {
    config: crate::config::Config,
    account_id: near_primitives::types::AccountId,
}

impl AccountStorageManagementContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<AccountStorageManagement as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.0,
            account_id: scope.account_id.clone().into(),
        })
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = AccountStorageManagementContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// What do you want to do with the storage?
pub enum StorageActions {
    #[strum_discriminants(strum(
        message = "view-storage-balance    - View storage balance for an account"
    ))]
    /// View storage balance for an account
    ViewStorageBalance(self::view_storage_balance::ContractAccountId),
    #[strum_discriminants(strum(
        message = "storage-deposit         - Make a storage deposit for the account"
    ))]
    /// Make a storage deposit for the account
    StorageDeposit,
    #[strum_discriminants(strum(
        message = "storage-withdraw        - Withdraw storage for the account"
    ))]
    /// Withdraw storage for the account
    StorageWithdraw,
}