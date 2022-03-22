use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod change_method;
mod view_method;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = ())]
pub struct OptionMethod {
    #[interactive_clap(subcommand)]
    method: Method,
}

impl OptionMethod {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.method.process(prepopulated_unsigned_transaction).await
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = ())]
///Choose your method
pub enum Method {
    #[strum_discriminants(strum(
        message = "Execute a change  method (construct a transaction with a function call)"
    ))]
    /// Specify a change method
    ChangeMethod(self::change_method::operation_mode::OperationMode),
    #[strum_discriminants(strum(
        message = "Execute a view method (read-only call, which does not require signing)"
    ))]
    /// Specify a view method
    ViewMethod(self::view_method::operation_mode::OperationMode),
}

impl Method {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self {
            Self::ChangeMethod(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
            Self::ViewMethod(operation_mode) => operation_mode.process().await,
        }
    }
}
