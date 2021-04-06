use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

pub mod construct_transaction_command;
pub mod generate_shell_completions_command;
pub mod utils_command;


#[derive(Debug, clap::Clap)]
pub enum CliTopLevelCommand {
    /// Prepare and, optionally, submit a new transaction
    ConstructTransaction(self::construct_transaction_command::operation_mode::CliOperationMode),
    /// Helpers
    Utils(self::utils_command::CliUtils),
    /// Use these to generate static shell completions
    GenerateShellCompletions(self::generate_shell_completions_command::CliGenerateShellCompletions),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum TopLevelCommand {
    #[strum_discriminants(strum(message = "Construct a new transaction"))]
    ConstructTransaction(self::construct_transaction_command::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "Helpers"))]
    Utils(self::utils_command::Utils),
}

impl From<CliTopLevelCommand> for TopLevelCommand {
    fn from(cli_top_level_command: CliTopLevelCommand) -> Self {
        match cli_top_level_command {
            CliTopLevelCommand::ConstructTransaction(cli_operation_mode) => {
                TopLevelCommand::ConstructTransaction(cli_operation_mode.into())
            }
            CliTopLevelCommand::Utils(cli_util) => {
                TopLevelCommand::Utils(cli_util.into())
            }
            CliTopLevelCommand::GenerateShellCompletions(_) => {
                unreachable!("This variant is handled in the main function")
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
            TopLevelCommandDiscriminants::ConstructTransaction => {
                CliTopLevelCommand::ConstructTransaction(Default::default())
            }
            TopLevelCommandDiscriminants::Utils => {
                CliTopLevelCommand::Utils(Default::default())
            }
        };
        Self::from(cli_top_level_command)
    }

    pub async fn process(self) -> crate::CliResult {
        match self {
            Self::ConstructTransaction(mode) => {
                let unsigned_transaction = near_primitives::transaction::Transaction {
                    signer_id: "".to_string(),
                    public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
                    nonce: 0,
                    receiver_id: "".to_string(),
                    block_hash: Default::default(),
                    actions: vec![],
                };
                mode.process(unsigned_transaction).await
            }
            Self::Utils(util_type) => util_type.process().await,
        }
    }
}
