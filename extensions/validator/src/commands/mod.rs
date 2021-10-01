use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

pub mod validators;
pub mod proposals;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliTopLevelCommand {
    /// Lookup validators for given epoch
    Validators(self::validators::operation_mode::CliOperationMode),
    /// Show both new proposals in the current epoch as well as current validators who are implicitly proposing
    Proposals(self::proposals::operation_mode::CliOperationMode),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum TopLevelCommand {
    #[strum_discriminants(strum(message = "Lookup validators for given epoch"))]
    Validators(self::validators::operation_mode::OperationMode),
    #[strum_discriminants(strum(
        message = "Show both new proposals in the current epoch as well as current validators who are implicitly proposing"
    ))]
    Proposals(self::proposals::operation_mode::OperationMode),
}

impl CliTopLevelCommand {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::Validators(subcommand) => {
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

impl From<TopLevelCommand> for CliTopLevelCommand {
    fn from(top_level_command: TopLevelCommand) -> Self {
        match top_level_command {
            TopLevelCommand::Validators(validators_request) => {
                Self::Validators(validators_request.into())
            }
            TopLevelCommand::Proposals(proposals_request) => Self::Proposals(proposals_request.into()),
        }
    }
}

impl From<CliTopLevelCommand> for TopLevelCommand {
    fn from(cli_top_level_command: CliTopLevelCommand) -> Self {
        match cli_top_level_command {
            CliTopLevelCommand::Validators(cli_validators_request) => {
                TopLevelCommand::Validators(cli_validators_request.into())
            }
            CliTopLevelCommand::Proposals(cli_proposals_request) => {
                TopLevelCommand::Proposals(cli_proposals_request.into())
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
            TopLevelCommandDiscriminants::Validators => {
                CliTopLevelCommand::Validators(Default::default())
            }
            TopLevelCommandDiscriminants::Proposals => {
                CliTopLevelCommand::Proposals(Default::default())
            }
        };
        Self::from(cli_top_level_command)
    }

    pub async fn process(self) -> crate::CliResult {
        match self {
            Self::Validators(validators_request) => validators_request.process().await,
            Self::Proposals(proposals_request) => proposals_request.process().await,
        }
    }
}
