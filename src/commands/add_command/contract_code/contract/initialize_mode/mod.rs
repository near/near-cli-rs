use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod call_function_type;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliNextAction {
    /// Add an initialize
    Initialize(self::call_function_type::CliCallFunctionAction),
    /// Don't add an initialize
    NoInitialize(CliNoInitialize),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum NextAction {
    #[strum_discriminants(strum(message = "Add an initialize"))]
    Initialize(self::call_function_type::CallFunctionAction),
    #[strum_discriminants(strum(message = "Don't add an initialize"))]
    NoInitialize(NoInitialize),
}

impl CliNextAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::Initialize(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("initialize".to_owned());
                args
            }
            Self::NoInitialize(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("no-initialize".to_owned());
                args
            }
        }
    }
}

impl From<NextAction> for CliNextAction {
    fn from(next_action: NextAction) -> Self {
        match next_action {
            NextAction::Initialize(call_function_action) => {
                Self::Initialize(call_function_action.into())
            }
            NextAction::NoInitialize(no_initialize) => Self::NoInitialize(no_initialize.into()),
        }
    }
}

impl NextAction {
    pub fn from(
        item: CliNextAction,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: String,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliNextAction::Initialize(cli_call_function_action) => Ok(NextAction::Initialize(
                self::call_function_type::CallFunctionAction::from(
                    cli_call_function_action,
                    connection_config,
                    sender_account_id,
                )?,
            )),
            CliNextAction::NoInitialize(cli_no_initialize) => Ok(NextAction::NoInitialize(
                NoInitialize::from(cli_no_initialize, connection_config, sender_account_id)?,
            )),
        }
    }
}

impl NextAction {
    pub fn choose_next_action(
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: String,
    ) -> color_eyre::eyre::Result<Self> {
        println!();
        let variants = NextActionDiscriminants::iter().collect::<Vec<_>>();
        let actions = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selected_action = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to choose next action")
            .items(&actions)
            .default(0)
            .interact()
            .unwrap();
        let cli_action = match variants[selected_action] {
            NextActionDiscriminants::Initialize => CliNextAction::Initialize(Default::default()),
            NextActionDiscriminants::NoInitialize => {
                CliNextAction::NoInitialize(Default::default())
            }
        };
        Ok(Self::from(
            cli_action,
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
            NextAction::Initialize(call_function_action) => {
                call_function_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            NextAction::NoInitialize(no_initialize) => {
                no_initialize
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

/// данные для инициализации
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliNoInitialize {
    #[clap(subcommand)]
    pub sign_option: Option<
        crate::commands::construct_transaction_command::sign_transaction::CliSignTransaction,
    >,
}

#[derive(Debug, Clone)]
pub struct NoInitialize {
    pub sign_option:
        crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
}

impl CliNoInitialize {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let args = self
            .sign_option
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        args
    }
}

impl From<NoInitialize> for CliNoInitialize {
    fn from(no_initialize: NoInitialize) -> Self {
        Self{sign_option: Some(crate::commands::construct_transaction_command::sign_transaction::CliSignTransaction::from(no_initialize.sign_option))}
    }
}

impl NoInitialize {
    fn from(
        item: CliNoInitialize,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: String,
    ) -> color_eyre::eyre::Result<Self> {
        let sign_option = match item.sign_option {
            Some(cli_sign_transaction) => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::from(cli_sign_transaction, connection_config, sender_account_id)?,
            None => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::choose_sign_option(connection_config, sender_account_id)?,
        };
        Ok(Self { sign_option })
    }
}

impl NoInitialize {
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
