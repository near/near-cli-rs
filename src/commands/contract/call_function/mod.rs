use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod as_read_only;
mod as_transaction;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct CallFunctionCommands {
    #[interactive_clap(subcommand)]
    call_function_actions: CallFunctionActions,
}

impl CallFunctionCommands {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        self.call_function_actions.process(config).await
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///Сhoose action for account
pub enum CallFunctionActions {
    #[strum_discriminants(strum(message = "as-read-only    - Calling a view method"))]
    ///Calling a view method
    AsReadOnly(self::as_read_only::CallFunctionView),
    #[strum_discriminants(strum(message = "as-transaction  - Calling a change method"))]
    ///Calling a change method
    AsTransaction(self::as_transaction::CallFunctionAction),
}

impl CallFunctionActions {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        match self {
            Self::AsReadOnly(call_function_view) => call_function_view.process(config).await,
            Self::AsTransaction(call_function_action) => call_function_action.process(config).await,
        }
    }
}
