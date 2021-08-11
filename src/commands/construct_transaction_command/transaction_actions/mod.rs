use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod add_access_key_mode;
mod call_function_type;
mod create_account_type;
mod delete_access_key_type;
mod delete_account_type;
mod stake_near_tokens_type;
mod transfer_near_tokens_type;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliNextAction {
    /// Choose next action
    AddAction(CliSelectAction),
    /// Go to transaction signing
    Skip(CliSkipAction),
}

#[derive(Debug, Clone, clap::Clap)]
pub enum CliSkipNextAction {
    /// Go to transaction signing
    Skip(CliSkipAction),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum NextAction {
    #[strum_discriminants(strum(message = "Select a new action"))]
    AddAction(SelectAction),
    #[strum_discriminants(strum(message = "Skip adding a new action"))]
    Skip(SkipAction),
}

impl CliSkipNextAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::Skip(subcommand) => {
                // let mut args = ;  it is not implemented now!!!
                // args.push_front("skip".to_owned());
                // args
                subcommand.to_cli_args()
            }
        }
    }
}

impl From<NextAction> for CliSkipNextAction {
    fn from(next_action: NextAction) -> Self {
        match next_action {
            NextAction::AddAction(_select_action) => {
                Self::Skip(CliSkipAction { sign_option: None })
            }
            NextAction::Skip(skip_action) => Self::Skip(skip_action.into()),
        }
    }
}

impl CliNextAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::AddAction(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("add-action".to_owned());
                args
            }
            Self::Skip(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("skip".to_owned());
                args
            }
        }
    }
}

impl From<NextAction> for CliNextAction {
    fn from(next_action: NextAction) -> Self {
        match next_action {
            NextAction::AddAction(select_action) => Self::AddAction(select_action.into()),
            NextAction::Skip(skip_action) => Self::Skip(skip_action.into()),
        }
    }
}

impl NextAction {
    pub fn from_cli_next_action(
        item: CliNextAction,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliNextAction::AddAction(cli_select_action) => {
                let select_action: SelectAction =
                    SelectAction::from(cli_select_action, connection_config, sender_account_id)?;
                Ok(Self::AddAction(select_action))
            }
            CliNextAction::Skip(cli_skip_action) => {
                let skip_action: SkipAction =
                    SkipAction::from(cli_skip_action, connection_config, sender_account_id)?;
                Ok(Self::Skip(skip_action))
            }
        }
    }
}

impl NextAction {
    pub fn from_cli_skip_next_action(
        item: CliSkipNextAction,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliSkipNextAction::Skip(cli_skip_action) => {
                let skip_action: SkipAction =
                    SkipAction::from(cli_skip_action, connection_config, sender_account_id)?;
                Ok(Self::Skip(skip_action))
            }
        }
    }
}

/// This mode is not provided now
// impl CliNextAction {
//     fn from(item: CliSkipNextAction) -> color_eyre::eyre::Result<Self> {
//         match item {
//             CliSkipNextAction::Skip(cli_skip_action) => Ok(Self::Skip(cli_skip_action)),
//         }
//     }
// }

impl NextAction {
    pub fn input_next_action(
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        println!();
        let variants = NextActionDiscriminants::iter().collect::<Vec<_>>();
        let next_action = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let select_next_action = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an action that you want to add to the action:")
            .items(&next_action)
            .default(0)
            .interact()
            .unwrap();
        let cli_next_action = match variants[select_next_action] {
            NextActionDiscriminants::AddAction => CliNextAction::AddAction(Default::default()),
            NextActionDiscriminants::Skip => CliNextAction::Skip(Default::default()),
        };
        Ok(Self::from_cli_next_action(
            cli_next_action,
            connection_config,
            sender_account_id,
        )?)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self {
            NextAction::AddAction(select_action) => {
                select_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            NextAction::Skip(skip_action) => {
                skip_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

/// инструмент для добавления команды в транзакцию
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliSelectAction {
    #[clap(subcommand)]
    transaction_subcommand: Option<CliActionSubcommand>,
}

#[derive(Debug, Clone)]
pub struct SelectAction {
    transaction_subcommand: ActionSubcommand,
}

impl CliSelectAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        self.transaction_subcommand
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default()
    }
}

impl From<SelectAction> for CliSelectAction {
    fn from(select_action: SelectAction) -> Self {
        Self {
            transaction_subcommand: Some(select_action.transaction_subcommand.into()),
        }
    }
}

impl SelectAction {
    fn from(
        item: CliSelectAction,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        let transaction_subcommand: ActionSubcommand = match item.transaction_subcommand {
            Some(cli_transaction_subcommand) => ActionSubcommand::from(
                cli_transaction_subcommand,
                connection_config,
                sender_account_id,
            ),
            None => ActionSubcommand::choose_action_command(connection_config, sender_account_id),
        };
        Ok(Self {
            transaction_subcommand,
        })
    }
}

impl SelectAction {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        self.transaction_subcommand
            .process(prepopulated_unsigned_transaction, network_connection_config)
            .await
    }
}

#[derive(Debug, Clone, clap::Clap)]
pub enum CliActionSubcommand {
    /// Предоставьте данные для перевода Near
    TransferNEARTokens(self::transfer_near_tokens_type::CliTransferNEARTokensAction),
    /// Предоставьте данные для call function
    CallFunction(self::call_function_type::CliCallFunctionAction),
    /// Предоставьте данные для ставки
    StakeNEARTokens(self::stake_near_tokens_type::CliStakeNEARTokensAction),
    /// Предоставьте данные для создания аккаунта
    CreateAccount(self::create_account_type::CliCreateAccountAction),
    /// Предоставьте данные для удаления аккаунта
    DeleteAccount(self::delete_account_type::CliDeleteAccountAction),
    /// Предоставьте данные для добавления ключа доступа пользователю
    AddAccessKey(self::add_access_key_mode::CliAddAccessKeyMode),
    /// Предоставьте данные для удаления ключа доступа у пользователя
    DeleteAccessKey(self::delete_access_key_type::CliDeleteAccessKeyAction),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum ActionSubcommand {
    #[strum_discriminants(strum(message = "Transfer NEAR Tokens"))]
    TransferNEARTokens(self::transfer_near_tokens_type::TransferNEARTokensAction),
    #[strum_discriminants(strum(message = "Call a Function"))]
    CallFunction(self::call_function_type::CallFunctionAction),
    #[strum_discriminants(strum(message = "Stake NEAR Tokens"))]
    StakeNEARTokens(self::stake_near_tokens_type::StakeNEARTokensAction),
    #[strum_discriminants(strum(message = "Create an Account"))]
    CreateAccount(self::create_account_type::CreateAccountAction),
    #[strum_discriminants(strum(message = "Delete an Account"))]
    DeleteAccount(self::delete_account_type::DeleteAccountAction),
    #[strum_discriminants(strum(message = "Add an Access Key"))]
    AddAccessKey(self::add_access_key_mode::AddAccessKeyMode),
    #[strum_discriminants(strum(message = "Detete an Access Key"))]
    DeleteAccessKey(self::delete_access_key_type::DeleteAccessKeyAction),
}

impl CliActionSubcommand {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::TransferNEARTokens(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("transfer-near-tokens".to_owned());
                args
            }
            Self::CallFunction(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("call-function".to_owned());
                args
            }
            Self::StakeNEARTokens(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("stake-near-tokens".to_owned());
                args
            }
            Self::CreateAccount(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("create-account".to_owned());
                args
            }
            Self::DeleteAccount(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("delete-account".to_owned());
                args
            }
            Self::AddAccessKey(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("add-access-key".to_owned());
                args
            }
            Self::DeleteAccessKey(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("delete-access-key".to_owned());
                args
            }
        }
    }
}

impl From<ActionSubcommand> for CliActionSubcommand {
    fn from(action_subcommand: ActionSubcommand) -> Self {
        match action_subcommand {
            ActionSubcommand::TransferNEARTokens(transfer_near_tokens_action) => {
                Self::TransferNEARTokens(transfer_near_tokens_action.into())
            }
            ActionSubcommand::CallFunction(call_function_action) => {
                Self::CallFunction(call_function_action.into())
            }
            ActionSubcommand::StakeNEARTokens(stake_near_tokens_action) => {
                Self::StakeNEARTokens(stake_near_tokens_action.into())
            }
            ActionSubcommand::CreateAccount(create_account_action) => {
                Self::CreateAccount(create_account_action.into())
            }
            ActionSubcommand::DeleteAccount(delete_account_action) => {
                Self::DeleteAccount(delete_account_action.into())
            }
            ActionSubcommand::AddAccessKey(add_access_key_action) => {
                Self::AddAccessKey(add_access_key_action.into())
            }
            ActionSubcommand::DeleteAccessKey(delete_access_key_action) => {
                Self::DeleteAccessKey(delete_access_key_action.into())
            }
        }
    }
}

impl ActionSubcommand {
    fn from(
        item: CliActionSubcommand,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> Self {
        match item {
            CliActionSubcommand::TransferNEARTokens(cli_transfer_near_token) => {
                Self::TransferNEARTokens(
                    self::transfer_near_tokens_type::TransferNEARTokensAction::from(
                        cli_transfer_near_token,
                        connection_config,
                        sender_account_id,
                    )
                    .unwrap(),
                )
            }
            CliActionSubcommand::CreateAccount(cli_create_account) => Self::CreateAccount(
                self::create_account_type::CreateAccountAction::from(
                    cli_create_account,
                    connection_config,
                    sender_account_id,
                )
                .unwrap(),
            ),
            CliActionSubcommand::DeleteAccount(cli_delete_account) => Self::DeleteAccount(
                self::delete_account_type::DeleteAccountAction::from(
                    cli_delete_account,
                    connection_config,
                    sender_account_id,
                )
                .unwrap(),
            ),
            CliActionSubcommand::AddAccessKey(cli_add_access_key) => Self::AddAccessKey(
                self::add_access_key_mode::AddAccessKeyMode::from(
                    cli_add_access_key,
                    connection_config,
                    sender_account_id,
                )
                .unwrap(),
            ),
            CliActionSubcommand::DeleteAccessKey(cli_delete_access_key) => Self::DeleteAccessKey(
                self::delete_access_key_type::DeleteAccessKeyAction::from(
                    cli_delete_access_key,
                    connection_config,
                    sender_account_id,
                )
                .unwrap(),
            ),
            CliActionSubcommand::StakeNEARTokens(cli_stake_near_token) => Self::StakeNEARTokens(
                self::stake_near_tokens_type::StakeNEARTokensAction::from(
                    cli_stake_near_token,
                    connection_config,
                    sender_account_id,
                )
                .unwrap(),
            ),
            CliActionSubcommand::CallFunction(cli_call_function) => Self::CallFunction(
                self::call_function_type::CallFunctionAction::from(
                    cli_call_function,
                    connection_config,
                    sender_account_id,
                )
                .unwrap(),
            ),
        }
    }
}

impl ActionSubcommand {
    pub fn choose_action_command(
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> ActionSubcommand {
        println!();
        let variants = ActionSubcommandDiscriminants::iter().collect::<Vec<_>>();
        let action_subcommands = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let select_action_subcommand = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an action that you want to add to the action:")
            .items(&action_subcommands)
            .default(0)
            .interact()
            .unwrap();
        let cli_action_subcomand = match variants[select_action_subcommand] {
            ActionSubcommandDiscriminants::TransferNEARTokens => {
                CliActionSubcommand::TransferNEARTokens(Default::default())
            }
            ActionSubcommandDiscriminants::CallFunction => {
                CliActionSubcommand::CallFunction(Default::default())
            }
            ActionSubcommandDiscriminants::StakeNEARTokens => {
                CliActionSubcommand::StakeNEARTokens(Default::default())
            }
            ActionSubcommandDiscriminants::CreateAccount => {
                CliActionSubcommand::CreateAccount(Default::default())
            }
            ActionSubcommandDiscriminants::DeleteAccount => {
                CliActionSubcommand::DeleteAccount(Default::default())
            }
            ActionSubcommandDiscriminants::AddAccessKey => {
                CliActionSubcommand::AddAccessKey(Default::default())
            }
            ActionSubcommandDiscriminants::DeleteAccessKey => {
                CliActionSubcommand::DeleteAccessKey(Default::default())
            }
        };
        Self::from(cli_action_subcomand, connection_config, sender_account_id)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self {
            ActionSubcommand::TransferNEARTokens(args_transfer) => {
                args_transfer
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            ActionSubcommand::CallFunction(args_function) => {
                args_function
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            ActionSubcommand::StakeNEARTokens(args_stake) => {
                args_stake
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            ActionSubcommand::CreateAccount(args_create_account) => {
                args_create_account
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            ActionSubcommand::DeleteAccount(args_delete_account) => {
                args_delete_account
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            ActionSubcommand::AddAccessKey(args_add_access_key) => {
                args_add_access_key
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            ActionSubcommand::DeleteAccessKey(args_delete_access_key) => {
                args_delete_access_key
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

/// инструмент, показывающий окончание набора команд в одной транзакции
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliSkipAction {
    #[clap(subcommand)]
    sign_option: Option<super::sign_transaction::CliSignTransaction>,
}

#[derive(Debug, Clone)]
pub struct SkipAction {
    pub sign_option: super::sign_transaction::SignTransaction,
}

impl CliSkipAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        self.sign_option
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default()
    }
}

impl From<SkipAction> for CliSkipAction {
    fn from(skip_action: SkipAction) -> Self {
        Self {
            sign_option: Some(skip_action.sign_option.into()),
        }
    }
}

// impl From<SelectAction> for CliSkipAction {
//     fn from(select_action: SelectAction) -> Self {
//         Self{
//             sign_option:
//         }
//     }
// }

impl SkipAction {
    fn from(
        item: CliSkipAction,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        let sign_option: super::sign_transaction::SignTransaction = match item.sign_option {
            Some(cli_sign_transaction) => super::sign_transaction::SignTransaction::from(
                cli_sign_transaction,
                connection_config,
                sender_account_id,
            )?,
            None => super::sign_transaction::SignTransaction::choose_sign_option(
                connection_config,
                sender_account_id,
            )?,
        };
        Ok(Self { sign_option })
    }
}

impl SkipAction {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self
            .sign_option
            .process(
                prepopulated_unsigned_transaction,
                network_connection_config.clone(),
            )
            .await?
        {
            Some(transaction_info) => {
                crate::common::print_transaction_status(
                    transaction_info,
                    network_connection_config,
                )
                .await;
            }
            None => {}
        };
        Ok(())
    }
}
