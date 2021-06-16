use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod call_function_type;

#[derive(Debug, clap::Clap)]
pub enum CliNextAction {
    /// Add an initialize
    Initialize(self::call_function_type::CliCallFunctionAction),
    /// Don't add an initialize
    NoInitialize(CliNoInitialize),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum NextAction {
    #[strum_discriminants(strum(message = "Add an initialize"))]
    Initialize(self::call_function_type::CallFunctionAction),
    #[strum_discriminants(strum(message = "Don't add an initialize"))]
    NoInitialize(NoInitialize),
}

impl From<CliNextAction> for NextAction {
    fn from(item: CliNextAction) -> Self {
        match item {
            CliNextAction::Initialize(cli_call_function_action) => {
                NextAction::Initialize(cli_call_function_action.into())
            }
            CliNextAction::NoInitialize(cli_no_initialize) => {
                NextAction::NoInitialize(cli_no_initialize.into())
            }
        }
    }
}

impl NextAction {
    pub fn choose_next_action() -> Self {
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
        Self::from(cli_action)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
        file_path: std::path::PathBuf,
    ) -> crate::CliResult {
        match self {
            NextAction::Initialize(call_function_action) => {
                call_function_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            NextAction::NoInitialize(no_initialize) => {
                no_initialize
                    .process(
                        prepopulated_unsigned_transaction,
                        network_connection_config,
                        file_path,
                    )
                    .await
            }
        }
    }
}

/// данные для инициализации
#[derive(Debug, Default, clap::Clap)]
pub struct CliNoInitialize {
    #[clap(subcommand)]
    pub sign_option: Option<
        crate::commands::construct_transaction_command::sign_transaction::CliSignTransaction,
    >,
}

#[derive(Debug)]
pub struct NoInitialize {
    pub sign_option:
        crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
}

impl From<CliNoInitialize> for NoInitialize {
    fn from(item: CliNoInitialize) -> Self {
        let sign_option = match item.sign_option {
            Some(cli_sign_transaction) => cli_sign_transaction.into(),
            None => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::choose_sign_option(),
        };
        Self { sign_option }
    }
}

impl NoInitialize {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
        file_path: std::path::PathBuf,
    ) -> crate::CliResult {
        match self
            .sign_option
            .process(prepopulated_unsigned_transaction, network_connection_config)
            .await?
        {
            Some(transaction_info) => {
                println!(
                    "\n Contract code {:?} has been successfully deployed.",
                    file_path
                );
                println!("\nTransaction Id {id}.\n\nTo see the transaction in the transaction explorer, please open this url in your browser:
                    \nhttps://explorer.testnet.near.org/transactions/{id}\n", id=transaction_info.transaction_outcome.id);
            }
            None => {}
        };
        Ok(())
    }
}
