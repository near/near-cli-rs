use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod access_key;
mod account;

/// инструмент выбора to delete action
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliDeleteAction {
    #[clap(subcommand)]
    action: Option<CliAction>,
}

#[derive(Debug)]
pub struct DeleteAction {
    pub action: Action,
}

impl From<CliDeleteAction> for DeleteAction {
    fn from(item: CliDeleteAction) -> Self {
        let action = match item.action {
            Some(cli_action) => cli_action.into(),
            None => Action::choose_action(),
        };
        Self { action }
    }
}

impl DeleteAction {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.action.process(prepopulated_unsigned_transaction).await
    }
}

#[derive(Debug, clap::Clap)]
pub enum CliAction {
    /// Delete an access key for an account
    AccessKey(self::access_key::operation_mode::CliOperationMode),
    /// Delete this account
    Account(self::account::operation_mode::CliOperationMode),
    // /// Add a new sub-account
    // SubAccount,
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Action {
    #[strum_discriminants(strum(message = "Delete an access key for this account"))]
    AccessKey(self::access_key::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "Delete this account"))]
    Account(self::account::operation_mode::OperationMode),
    // SubAccount,
}

impl From<CliAction> for Action {
    fn from(item: CliAction) -> Self {
        match item {
            CliAction::AccessKey(cli_operation_mode) => {
                Action::AccessKey(cli_operation_mode.into())
            }
            CliAction::Account(cli_operation_mode) => Action::Account(cli_operation_mode.into()),
        }
    }
}

impl Action {
    fn choose_action() -> Self {
        println!();
        let variants = ActionDiscriminants::iter().collect::<Vec<_>>();
        let actions = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selected_action = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Сhoose what you want to delete")
            .items(&actions)
            .default(0)
            .interact()
            .unwrap();
        let cli_action = match variants[selected_action] {
            ActionDiscriminants::AccessKey => CliAction::AccessKey(Default::default()),
            ActionDiscriminants::Account => CliAction::Account(Default::default()),
        };
        Self::from(cli_action)
    }

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
