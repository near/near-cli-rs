use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

pub mod view_command;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliTopLevelCommand {
    /// View account, contract code, contract state, transaction, nonce, recent block hash
    View(self::view_command::CliViewQueryRequest),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum TopLevelCommand {
    #[strum_discriminants(strum(
        message = "View account, contract code, contract state, transaction, nonce, recent block hash"
    ))]
    View(self::view_command::ViewQueryRequest),
}

impl CliTopLevelCommand {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::View(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("view".to_owned());
                args
            }
        }
    }
}

impl From<TopLevelCommand> for CliTopLevelCommand {
    fn from(top_level_command: TopLevelCommand) -> Self {
        match top_level_command {
            TopLevelCommand::View(view_query_request) => Self::View(view_query_request.into()),
        }
    }
}

impl From<CliTopLevelCommand> for TopLevelCommand {
    fn from(cli_top_level_command: CliTopLevelCommand) -> Self {
        match cli_top_level_command {
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
            TopLevelCommandDiscriminants::View => CliTopLevelCommand::View(Default::default()),
        };
        Self::from(cli_top_level_command)
    }

    pub async fn process(self) -> crate::CliResult {
        match self {
            Self::View(view_query_request) => view_query_request.process().await,
        }
    }
}
