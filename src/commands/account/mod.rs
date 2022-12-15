use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod add_key;
mod create_account;
mod delete_account;
mod delete_key;
mod import_account;
mod list_keys;
mod view_account_summary;

pub const MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH: usize = 32;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct AccountCommands {
    #[interactive_clap(subcommand)]
    account_actions: AccountActions,
}

impl AccountCommands {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        self.account_actions.process(config).await
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
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
}

impl AccountActions {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        match self {
            Self::ViewAccountSummary(view_account_command) => {
                view_account_command.process(config).await
            }
            Self::ListKeys(view_list_keys) => view_list_keys.process(config).await,
            Self::DeleteAccount(delete_account) => delete_account.process(config).await,
            Self::CreateAccount(account) => account.process(config).await,
            Self::AddKey(add_key_command) => add_key_command.process(config).await,
            Self::DeleteKey(delete_key_command) => delete_key_command.process(config).await,
            Self::ImportAccount(import_account_command) => {
                import_account_command.process(config).await
            }
        }
    }
}
