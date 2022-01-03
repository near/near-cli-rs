use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod proposals;
pub mod validators;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = ())]
///Select command
pub enum TopLevelCommand {
    #[strum_discriminants(strum(message = "Lookup validators for given epoch"))]
    /// Lookup validators for given epoch
    Validators(self::validators::operation_mode::OperationMode),
    #[strum_discriminants(strum(
        message = "Show both new proposals in the current epoch as well as current validators who are implicitly proposing"
    ))]
    /// Show both new proposals in the current epoch as well as current validators who are implicitly proposing
    Proposals(self::proposals::operation_mode::OperationMode),
}

impl TopLevelCommand {
    pub async fn process(self) -> crate::CliResult {
        match self {
            Self::Validators(validators_request) => validators_request.process().await,
            Self::Proposals(proposals_request) => proposals_request.process().await,
        }
    }
}
