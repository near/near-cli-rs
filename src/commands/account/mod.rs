use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod add_key;
pub mod create_account;
mod delete_account;
mod delete_key;
mod import_account;
mod list_keys;
pub mod profile_management;
pub mod storage_management;
mod view_account_summary;

pub const MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH: usize = 32;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct AccountCommands {
    #[interactive_clap(subcommand)]
    account_actions: AccountActions,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// What do you want to do with an account?
pub enum AccountActions {
    #[strum_discriminants(strum(
        message = "view-account-summary    - View properties for an account"
    ))]
    /// View properties for an account
    ViewAccountSummary(self::view_account_summary::ViewAccountSummary),
    #[strum_discriminants(strum(
        message = "import-account          - Import existing account (a.k.a. \"sign in\")"
    ))]
    /// Import existing account (a.k.a. "sign in")
    ImportAccount(self::import_account::ImportAccountCommand),
    #[strum_discriminants(strum(message = "create-account          - Create a new account"))]
    /// Create a new account
    CreateAccount(self::create_account::CreateAccount),
    #[strum_discriminants(strum(message = "delete-account          - Delete an account"))]
    /// Delete an account
    DeleteAccount(self::delete_account::DeleteAccount),
    #[strum_discriminants(strum(
        message = "list-keys               - View a list of access keys of an account"
    ))]
    /// View a list of access keys of an account
    ListKeys(self::list_keys::ViewListKeys),
    #[strum_discriminants(strum(
        message = "add-key                 - Add an access key to an account"
    ))]
    /// Add an access key to an account
    AddKey(self::add_key::AddKeyCommand),
    #[strum_discriminants(strum(
        message = "delete-key              - Delete an access key from an account"
    ))]
    /// Delete an access key from an account
    DeleteKey(self::delete_key::DeleteKeyCommand),
    #[strum_discriminants(strum(
        message = "manage-profile          - Profile management: view, update"
    ))]
    /// Profile management: view, update
    ManageProfile(self::profile_management::Contract),
    #[strum_discriminants(strum(
        message = "manage-storage-deposit  - Storage management: deposit, withdrawal, balance review"
    ))]
    /// Storage management for contract: deposit, withdrawal, balance review
    ManageStorageDeposit(self::storage_management::Contract),
}
