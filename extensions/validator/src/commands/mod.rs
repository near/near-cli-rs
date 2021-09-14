// use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

pub mod list_command;
pub mod proposals_command;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliValidatorCommand {
    /// TODO: add description
    List(self::list_command::CliListAction),
    /// Prepare and, optionally, submit a new transaction
    Proposals(self::proposals_command::CliProposalsAction),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum ValidatorCommand {
    #[strum_discriminants(strum(message = "TODO: add list command description"))]
    List(self::list_command::ListAction),
    #[strum_discriminants(strum(message = "TODO: add proposals command description"))]
    Proposals(self::proposals_command::ProposalsAction),
}

impl CliValidatorCommand {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::List(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("list".to_owned());
                args
            }
            Self::Proposals(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("proposals".to_owned());
                args
            }
        }
    }
}

impl From<ValidatorCommand> for CliValidatorCommand {
    fn from(validator_command: ValidatorCommand) -> Self {
        match validator_command {
            ValidatorCommand::List(epoch) => Self::List(epoch.into()),
            ValidatorCommand::Proposals() => Self::Proposals()
        }
    }
}

impl From<CliValidatorCommand> for ValidatorCommand {
    fn from(cli_validator_command: CliValidatorCommand) -> Self {
        match cli_validator_command {
            CliValidatorCommand::List(cli_list_action) => {
                ValidatorCommand::List(self::list_command::List::from(cli_list_action).unwrap())
            }
            CliValidatorCommand::Proposals(cli_proposals_action) => {
                ValidatorCommand::ProposalsAction(
                    self::proposals_command::ProposalsAction::from(
                        cli_proposals_action,
                    )
                    .unwrap(),
                )
            }
        }
    }
}

impl ValidatorCommand {
    pub fn choose_command() -> Self {
        println!();
        let variants = TopLevelCommandDiscriminants::iter().collect::<Vec<_>>(); //TODO: what is TopLevelCommandDiscriminants?
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
        let cli_validator_command = match variants[selection] {
            TopLevelCommandDiscriminants::List => CliValidatorCommand::List(Default::default()),
            TopLevelCommandDiscriminants::Proposals => {
                CliValidatorCommand::Proposals(Default::default())
            }
        };
        Self::from(cli_validator_command)
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
            Self::List(list_action) => list_action.process(unsigned_transaction).await,
            Self::Proposals(proposals_action) => proposals_action.process(unsigned_transaction).await,
        }
    }
}
