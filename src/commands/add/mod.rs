use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod access_key;
mod contract_code;


/// инструмент выбора to add action
#[derive(Debug, Default, clap::Clap)]
pub struct CliAddAction {
    #[clap(subcommand)]
    action: Option<CliAction>,
}

#[derive(Debug)]
pub struct AddAction {
    pub action: Action,
}

impl From<CliAddAction> for AddAction {
    fn from(item: CliAddAction) -> Self {
        let action = match item.action {
            Some(cli_action) => cli_action.into(),
            None => Action::choose_action()
        };
        Self { action }
    }
}

impl AddAction {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.action
            .process(prepopulated_unsigned_transaction)
            .await
    }
}

#[derive(Debug, clap::Clap)]
pub enum CliAction {
    /// Add a new access key for an account
    AccessKey(self::access_key::operation_mode::CliOperationMode),
    /// Add a new contract code
    ContractCode(self::contract_code::operation_mode::CliOperationMode),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Action {
    #[strum_discriminants(strum(message = "Add access key"))]
    AccessKey(self::access_key::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "Add contract code"))]
    ContractCode(self::contract_code::operation_mode::OperationMode),
}

impl From<CliAction> for Action {
    fn from(item: CliAction) -> Self {
        match item {
            CliAction::AccessKey(cli_operation_mode) => {
                Action::AccessKey(cli_operation_mode.into())
            }
            CliAction::ContractCode(cli_operation_mode) => {
                Action::ContractCode(cli_operation_mode.into())
            }
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
            .with_prompt(
                "To construct a transaction you will need to provide information about sender (signer) and receiver accounts, and actions that needs to be performed.
                 \nDo you want to derive some information required for transaction construction automatically querying it online?"
            )
            .items(&actions)
            .default(0)
            .interact()
            .unwrap();
        let cli_action = match variants[selected_action] {
            ActionDiscriminants::AccessKey => CliAction::AccessKey(Default::default()),
            ActionDiscriminants::ContractCode => CliAction::ContractCode(Default::default()),
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
            Action::ContractCode(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await                
            }
        }
        
    }
}
