use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod add_access_key_mode;
mod add_contract_code_type;
mod call_function_type;
mod create_account_type;
mod delete_access_key_type;
mod delete_account_type;
mod stake_near_tokens_type;
mod transfer_near_tokens_type;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliSkipNextAction {
    /// Go to transaction signing
    Skip(CliSkipAction),
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = crate::common::SignerContext)]
///Select an action that you want to add to the action:
pub enum NextAction {
    #[strum_discriminants(strum(message = "Select a new action"))]
    /// Choose next action
    AddAction(SelectAction),
    #[strum_discriminants(strum(message = "Skip adding a new action"))]
    /// Go to transaction signing
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

impl NextAction {
    pub fn from_cli_skip_next_action(
        item: CliSkipNextAction,
        context: crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliSkipNextAction::Skip(cli_skip_action) => {
                let skip_action: SkipAction = SkipAction::from_cli(Some(cli_skip_action), context)?;
                Ok(Self::Skip(skip_action))
            }
        }
    }
}
//-------------------------------------
/// This mode is not provided now
// impl CliNextAction {
//     fn from(item: CliSkipNextAction) -> color_eyre::eyre::Result<Self> {
//         match item {
//             CliSkipNextAction::Skip(cli_skip_action) => Ok(Self::Skip(cli_skip_action)),
//         }
//     }
// }
//--------------------------------------
impl NextAction {
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

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SignerContext)]
pub struct SelectAction {
    #[interactive_clap(subcommand)]
    transaction_subcommand: ActionSubcommand,
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

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = crate::common::SignerContext)]
///Select an action that you want to add to the action:
pub enum ActionSubcommand {
    #[strum_discriminants(strum(message = "Transfer NEAR Tokens"))]
    /// Предоставьте данные для перевода Near
    TransferNearTokens(self::transfer_near_tokens_type::TransferNEARTokensAction),
    #[strum_discriminants(strum(message = "Call a Function"))]
    /// Предоставьте данные для call function
    CallFunction(self::call_function_type::CallFunctionAction),
    #[strum_discriminants(strum(message = "Stake NEAR Tokens"))]
    /// Предоставьте данные для ставки
    StakeNearTokens(self::stake_near_tokens_type::StakeNEARTokensAction),
    #[strum_discriminants(strum(message = "Create an Account"))]
    /// Предоставьте данные для создания аккаунта
    CreateAccount(self::create_account_type::CreateAccountAction),
    #[strum_discriminants(strum(message = "Delete an Account"))]
    /// Предоставьте данные для удаления аккаунта
    DeleteAccount(self::delete_account_type::DeleteAccountAction),
    #[strum_discriminants(strum(message = "Add an Access Key"))]
    /// Предоставьте данные для добавления ключа доступа пользователю
    AddAccessKey(self::add_access_key_mode::AddAccessKeyMode),
    #[strum_discriminants(strum(message = "Detete an Access Key"))]
    /// Предоставьте данные для удаления ключа доступа у пользователя
    DeleteAccessKey(self::delete_access_key_type::DeleteAccessKeyAction),
    #[strum_discriminants(strum(message = "Add a contract code"))]
    /// Предоставьте данные для добавления контракта
    AddContractCode(self::add_contract_code_type::ContractFile),
}

impl ActionSubcommand {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self {
            ActionSubcommand::TransferNearTokens(args_transfer) => {
                args_transfer
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            ActionSubcommand::CallFunction(args_function) => {
                args_function
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            ActionSubcommand::StakeNearTokens(args_stake) => {
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
            ActionSubcommand::AddContractCode(args_contract_file) => {
                args_contract_file
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SignerContext)]
pub struct SkipAction {
    #[interactive_clap(subcommand)]
    pub sign_option: super::sign_transaction::SignTransaction,
}
//------------------------------------
// impl From<SelectAction> for CliSkipAction {
//     fn from(select_action: SelectAction) -> Self {
//         Self{
//             sign_option:
//         }
//     }
// }
//-----------------------------------------

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
                );
            }
            None => {}
        };
        Ok(())
    }
}
