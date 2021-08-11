use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

pub mod add_command;
pub mod construct_transaction_command;
pub mod delete_command;
pub mod execute_command;
pub mod generate_shell_completions_command;
pub mod login;
pub mod transfer_command;
pub mod utils_command;
pub mod view_command;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliTopLevelCommand {
    /// Use these to add access key, contract code, stake proposal, sub-account, implicit-account
    Add(self::add_command::CliAddAction),
    /// Prepare and, optionally, submit a new transaction
    ConstructTransaction(self::construct_transaction_command::operation_mode::CliOperationMode),
    /// Use these to delete access key, sub-account
    Delete(self::delete_command::CliDeleteAction),
    /// Execute function (contract method)
    Execute(self::execute_command::CliOptionMethod),
    /// Use these to generate static shell completions
    GenerateShellCompletions(self::generate_shell_completions_command::CliGenerateShellCompletions),
    /// Use these to login with wallet authorization
    Login(self::login::operation_mode::CliOperationMode),
    /// Use these to transfer tokens
    Transfer(self::transfer_command::CliCurrency),
    /// Helpers
    Utils(self::utils_command::CliUtils),
    /// View account, contract code, contract state, transaction, nonce, recent block hash
    View(self::view_command::CliViewQueryRequest),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum TopLevelCommand {
    #[strum_discriminants(strum(message = "Login with wallet authorization"))]
    Login(self::login::operation_mode::OperationMode),
    #[strum_discriminants(strum(
        message = "View account, contract code, contract state, transaction, nonce, recent block hash"
    ))]
    View(self::view_command::ViewQueryRequest),
    #[strum_discriminants(strum(message = "Transfer tokens"))]
    Transfer(self::transfer_command::Currency),
    #[strum_discriminants(strum(message = "Execute function (contract method)"))]
    Execute(self::execute_command::OptionMethod),
    #[strum_discriminants(strum(
        message = "Add access key, contract code, stake proposal, sub-account, implicit-account"
    ))]
    Add(self::add_command::AddAction),
    #[strum_discriminants(strum(message = "Delete access key, account"))]
    Delete(self::delete_command::DeleteAction),
    #[strum_discriminants(strum(message = "Construct a new transaction"))]
    ConstructTransaction(self::construct_transaction_command::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "Helpers"))]
    Utils(self::utils_command::Utils),
}

impl CliTopLevelCommand {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::Login(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("login".to_owned());
                args
            }
            Self::Execute(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("execute".to_owned());
                args
            }
            Self::Add(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("add".to_owned());
                args
            }
            Self::Delete(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("delete".to_owned());
                args
            }
            Self::Transfer(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("transfer".to_owned());
                args
            }
            Self::View(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("view".to_owned());
                args
            }
            Self::ConstructTransaction(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("construct-transaction".to_owned());
                args
            }
            Self::Utils(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("utils".to_owned());
                args
            }
            Self::GenerateShellCompletions(_) => std::collections::VecDeque::new(),
        }
    }
}

impl From<TopLevelCommand> for CliTopLevelCommand {
    fn from(top_level_command: TopLevelCommand) -> Self {
        match top_level_command {
            TopLevelCommand::Login(operation_mode) => Self::Login(operation_mode.into()),
            TopLevelCommand::Execute(option_method) => Self::Execute(option_method.into()),
            TopLevelCommand::Add(add_action) => Self::Add(add_action.into()),
            TopLevelCommand::Delete(delete_action) => Self::Delete(delete_action.into()),
            TopLevelCommand::Transfer(currency) => Self::Transfer(currency.into()),
            TopLevelCommand::View(view_query_request) => Self::View(view_query_request.into()),
            TopLevelCommand::ConstructTransaction(operation_mode) => {
                Self::ConstructTransaction(operation_mode.into())
            }
            TopLevelCommand::Utils(utils) => Self::Utils(utils.into()),
        }
    }
}

impl From<CliTopLevelCommand> for TopLevelCommand {
    fn from(cli_top_level_command: CliTopLevelCommand) -> Self {
        match cli_top_level_command {
            CliTopLevelCommand::Add(cli_add_action) => {
                TopLevelCommand::Add(self::add_command::AddAction::from(cli_add_action).unwrap())
            }
            CliTopLevelCommand::ConstructTransaction(cli_operation_mode) => {
                TopLevelCommand::ConstructTransaction(
                    self::construct_transaction_command::operation_mode::OperationMode::from(
                        cli_operation_mode,
                    )
                    .unwrap(),
                )
            }
            CliTopLevelCommand::Delete(cli_delete_action) => TopLevelCommand::Delete(
                self::delete_command::DeleteAction::from(cli_delete_action).unwrap(),
            ),
            CliTopLevelCommand::Execute(cli_option_method) => TopLevelCommand::Execute(
                self::execute_command::OptionMethod::from(cli_option_method).unwrap(),
            ),
            CliTopLevelCommand::GenerateShellCompletions(_) => {
                unreachable!("This variant is handled in the main function")
            }
            CliTopLevelCommand::Login(cli_option_method) => {
                TopLevelCommand::Login(cli_option_method.into())
            }
            CliTopLevelCommand::Transfer(cli_currency) => TopLevelCommand::Transfer(
                self::transfer_command::Currency::from(cli_currency).unwrap(),
            ),
            CliTopLevelCommand::Utils(cli_util) => TopLevelCommand::Utils(cli_util.into()),
            CliTopLevelCommand::View(cli_view_query_request) => {
                TopLevelCommand::View(cli_view_query_request.into())
            }
        }
    }
}

impl TopLevelCommand {
    pub fn choose_command() -> Self {
        println!();
        let variants = TopLevelCommandDiscriminants::iter().collect::<Vec<_>>();
        let commands = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose your action")
            .items(&commands)
            .default(0)
            .interact()
            .unwrap();
        let cli_top_level_command = match variants[selection] {
            TopLevelCommandDiscriminants::Add => CliTopLevelCommand::Add(Default::default()),
            TopLevelCommandDiscriminants::ConstructTransaction => {
                CliTopLevelCommand::ConstructTransaction(Default::default())
            }
            TopLevelCommandDiscriminants::Delete => CliTopLevelCommand::Delete(Default::default()),
            TopLevelCommandDiscriminants::Execute => {
                CliTopLevelCommand::Execute(Default::default())
            }
            TopLevelCommandDiscriminants::Login => CliTopLevelCommand::Login(Default::default()),
            TopLevelCommandDiscriminants::Transfer => {
                CliTopLevelCommand::Transfer(Default::default())
            }
            TopLevelCommandDiscriminants::Utils => CliTopLevelCommand::Utils(Default::default()),
            TopLevelCommandDiscriminants::View => CliTopLevelCommand::View(Default::default()),
        };
        Self::from(cli_top_level_command)
    }

    pub async fn process(self) -> crate::CliResult {
        let unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: near_primitives::types::AccountId::test_account(),
            public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
            nonce: 0,
            receiver_id: near_primitives::types::AccountId::test_account(),
            block_hash: Default::default(),
            actions: vec![],
        };
        match self {
            Self::Add(add_action) => add_action.process(unsigned_transaction).await,
            Self::ConstructTransaction(mode) => mode.process(unsigned_transaction).await,
            Self::Delete(delete_action) => delete_action.process(unsigned_transaction).await,
            Self::Execute(option_method) => option_method.process(unsigned_transaction).await,
            Self::Login(mode) => mode.process().await,
            Self::Transfer(currency) => currency.process(unsigned_transaction).await,
            Self::Utils(util_type) => util_type.process().await,
            Self::View(view_query_request) => view_query_request.process().await,
        }
    }
}
