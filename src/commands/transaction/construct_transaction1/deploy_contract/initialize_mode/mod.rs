use async_recursion::async_recursion;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod call_function_type;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///Select the need for initialization
pub enum InitializeMode {
    /// Add an initialize
    #[strum_discriminants(strum(message = "with-init-call     - Add an initialize"))]
    WithInitCall(self::call_function_type::CallFunctionAction),
    /// Don't add an initialize
    #[strum_discriminants(strum(message = "without-init-call  - Don't add an initialize"))]
    WithoutInitCall(NoInitialize),
}

impl InitializeMode {
    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self {
            InitializeMode::WithInitCall(call_function_action) => {
                call_function_action
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
            InitializeMode::WithoutInitCall(no_initialize) => {
                no_initialize
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct NoInitialize {
    #[interactive_clap(subcommand)]
    next_action: super::super::BoxNextAction,
}

impl NoInitialize {
    #[async_recursion(?Send)]
    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match *self.next_action.clone().inner {
            super::super::NextAction::AddAction(select_action) => {
                select_action
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
            super::super::NextAction::Skip(skip_action) => {
                skip_action
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
        }
    }
}
