use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod call_function_type;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = crate::common::SignerContext)]
pub enum NextAction {
    /// Add an initialize
    #[strum_discriminants(strum(message = "Add an initialize"))]
    Initialize(self::call_function_type::CallFunctionAction),
    /// Don't add an initialize
    #[strum_discriminants(strum(message = "Don't add an initialize"))]
    NoInitialize(NoInitialize),
}

impl NextAction {
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

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SignerContext)]
pub struct NoInitialize {
    #[interactive_clap(subcommand)]
    pub sign_option:
        crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
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
                );
            }
            None => {}
        };
        Ok(())
    }
}
