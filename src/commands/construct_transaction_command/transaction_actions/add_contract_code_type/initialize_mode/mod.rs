use async_recursion::async_recursion;
use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod call_function_type;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliContractMode {
    /// Add an initialize
    Initialize(self::call_function_type::CliCallFunctionAction),
    /// Don't add an initialize
    NoInitialize(CliNoInitialize),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum ContractMode {
    #[strum_discriminants(strum(message = "Add an initialize"))]
    Initialize(self::call_function_type::CallFunctionAction),
    #[strum_discriminants(strum(message = "Don't add an initialize"))]
    NoInitialize(NoInitialize),
}

impl CliContractMode {
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

impl From<ContractMode> for CliContractMode {
    fn from(contract_mode: ContractMode) -> Self {
        match contract_mode {
            ContractMode::Initialize(call_function_action) => {
                Self::Initialize(call_function_action.into())
            }
            ContractMode::NoInitialize(no_initialize) => Self::NoInitialize(no_initialize.into()),
        }
    }
}

impl ContractMode {
    pub fn from(
        item: CliContractMode,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliContractMode::Initialize(cli_call_function_action) => Ok(ContractMode::Initialize(
                self::call_function_type::CallFunctionAction::from(
                    cli_call_function_action,
                    connection_config,
                    sender_account_id,
                )?,
            )),
            CliContractMode::NoInitialize(cli_no_initialize) => Ok(ContractMode::NoInitialize(
                NoInitialize::from(cli_no_initialize, connection_config, sender_account_id)?,
            )),
        }
    }
}

impl ContractMode {
    pub fn choose_contract_mode(
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        println!();
        let variants = ContractModeDiscriminants::iter().collect::<Vec<_>>();
        let actions = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selected_action = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Which contract mode do you want to choose?")
            .items(&actions)
            .default(0)
            .interact()
            .unwrap();
        let cli_action = match variants[selected_action] {
            ContractModeDiscriminants::Initialize => {
                CliContractMode::Initialize(Default::default())
            }
            ContractModeDiscriminants::NoInitialize => {
                CliContractMode::NoInitialize(Default::default())
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
            ContractMode::Initialize(call_function_action) => {
                call_function_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            ContractMode::NoInitialize(no_initialize) => {
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
    next_action: Option<super::super::CliSkipNextAction>,
}

#[derive(Debug, Clone)]
pub struct NoInitialize {
    pub next_action: Box<super::super::NextAction>,
}

impl CliNoInitialize {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        self.next_action
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default()
    }
}

impl From<NoInitialize> for CliNoInitialize {
    fn from(_no_initialize: NoInitialize) -> Self {
        Self {
            next_action: Some(super::super::CliSkipNextAction::Skip(
                super::super::CliSkipAction { sign_option: None },
            )),
        }
    }
}

impl NoInitialize {
    fn from(
        item: CliNoInitialize,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        let skip_next_action: super::super::NextAction = match item.next_action {
            Some(cli_skip_action) => super::super::NextAction::from_cli_skip_next_action(
                cli_skip_action,
                connection_config,
                sender_account_id,
            )?,
            None => {
                super::super::NextAction::input_next_action(connection_config, sender_account_id)?
            }
        };
        Ok(Self {
            next_action: Box::new(skip_next_action),
        })
    }
}

impl NoInitialize {
    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match *self.next_action {
            super::super::NextAction::AddAction(select_action) => {
                select_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            super::super::NextAction::Skip(skip_action) => {
                skip_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}
