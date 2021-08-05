use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod access_key;
mod contract_code;
mod implicit_account;
mod stake_proposal;
mod sub_account;

/// инструмент выбора to add action
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliAddAction {
    #[clap(subcommand)]
    action: Option<CliAction>,
}

#[derive(Debug, Clone)]
pub struct AddAction {
    pub action: Action,
}

impl CliAddAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        self.action
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default()
    }
}

impl From<AddAction> for CliAddAction {
    fn from(item: AddAction) -> Self {
        Self {
            action: Some(item.action.into()),
        }
    }
}

impl AddAction {
    pub fn from(item: CliAddAction) -> color_eyre::eyre::Result<Self> {
        let action = match item.action {
            Some(cli_action) => Action::from(cli_action)?,
            None => Action::choose_action()?,
        };
        Ok(Self { action })
    }
}

impl AddAction {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.action.process(prepopulated_unsigned_transaction).await
    }
}

#[derive(Debug, Clone, clap::Clap)]
pub enum CliAction {
    /// Add a new contract code
    ContractCode(self::contract_code::operation_mode::CliOperationMode),
    /// Add an implicit-account
    ImplicitAccount(self::implicit_account::CliImplicitAccount),
    /// Add a new stake proposal
    StakeProposal(self::stake_proposal::operation_mode::CliOperationMode),
    /// Add a new sub-account
    SubAccount(self::sub_account::operation_mode::CliOperationMode),
    /// Add a new access key for an account
    AccessKey(self::access_key::operation_mode::CliOperationMode),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Action {
    #[strum_discriminants(strum(message = "Add a new access key for an account"))]
    AccessKey(self::access_key::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "Add a new contract code"))]
    ContractCode(self::contract_code::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "Add an implicit-account"))]
    ImplicitAccount(self::implicit_account::ImplicitAccount),
    #[strum_discriminants(strum(message = "Add a new stake proposal"))]
    StakeProposal(self::stake_proposal::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "Add a new sub-account"))]
    SubAccount(self::sub_account::operation_mode::OperationMode),
}

impl CliAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::ContractCode(subcommand) => {
                let mut command = subcommand.to_cli_args();
                command.push_front("contract-code".to_owned());
                command
            }
            Self::AccessKey(subcommand) => {
                let mut command = subcommand.to_cli_args();
                command.push_front("access-key".to_owned());
                command
            }
            Self::ImplicitAccount(subcommand) => {
                let mut command = subcommand.to_cli_args();
                command.push_front("implicit-account".to_owned());
                command
            }
            _ => todo!(),
        }
    }
}

impl From<Action> for CliAction {
    fn from(item: Action) -> Self {
        match item {
            Action::ContractCode(operation_mode) => Self::ContractCode(operation_mode.into()),
            Action::AccessKey(operation_mode) => Self::AccessKey(operation_mode.into()),
            Action::ImplicitAccount(implicit_account) => {
                Self::ImplicitAccount(implicit_account.into())
            }
            _ => todo!(),
        }
    }
}

impl Action {
    fn from(item: CliAction) -> color_eyre::eyre::Result<Self> {
        match item {
            CliAction::AccessKey(cli_operation_mode) => Ok(Action::AccessKey(
                self::access_key::operation_mode::OperationMode::from(cli_operation_mode)?,
            )),
            CliAction::ContractCode(cli_operation_mode) => Ok(Action::ContractCode(
                self::contract_code::operation_mode::OperationMode::from(cli_operation_mode)
                    .unwrap(),
            )),
            CliAction::ImplicitAccount(cli_generate_keypair) => {
                Ok(Action::ImplicitAccount(cli_generate_keypair.into()))
            }
            CliAction::StakeProposal(cli_operation_mode) => Ok(Action::StakeProposal(
                self::stake_proposal::operation_mode::OperationMode::from(cli_operation_mode)
                    .unwrap(),
            )),
            CliAction::SubAccount(cli_operation_mode) => Ok(Action::SubAccount(
                self::sub_account::operation_mode::OperationMode::from(cli_operation_mode).unwrap(),
            )),
        }
    }
}

impl Action {
    fn choose_action() -> color_eyre::eyre::Result<Self> {
        println!();
        let variants = ActionDiscriminants::iter().collect::<Vec<_>>();
        let actions = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selected_action = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Сhoose what you want to add")
            .items(&actions)
            .default(0)
            .interact()
            .unwrap();
        let cli_action = match variants[selected_action] {
            ActionDiscriminants::AccessKey => CliAction::AccessKey(Default::default()),
            ActionDiscriminants::ContractCode => CliAction::ContractCode(Default::default()),
            ActionDiscriminants::ImplicitAccount => CliAction::ImplicitAccount(Default::default()),
            ActionDiscriminants::StakeProposal => CliAction::StakeProposal(Default::default()),
            ActionDiscriminants::SubAccount => CliAction::SubAccount(Default::default()),
        };
        Ok(Self::from(cli_action)?)
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
            Action::ImplicitAccount(generate_keypair) => generate_keypair.process().await,
            Action::StakeProposal(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
            Action::SubAccount(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
        }
    }
}
