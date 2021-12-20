use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod offline_mode;
mod online_mode;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = ())]
pub struct OperationMode {
    #[interactive_clap(subcommand)]
    pub mode: Mode,
}

impl OperationMode {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.mode.process(prepopulated_unsigned_transaction).await
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = ())]
///To construct a transaction you will need to provide information about sender (signer) and receiver accounts, and actions that needs to be performed.
///Do you want to derive some information required for transaction construction automatically querying it online?
pub enum Mode {
    #[strum_discriminants(strum(message = "Yes, I keep it simple"))]
    /// Prepare and, optionally, submit a new transaction with online mode
    Network(self::online_mode::NetworkArgs),
    #[strum_discriminants(strum(
        message = "No, I want to work in no-network (air-gapped) environment"
    ))]
    /// Prepare and, optionally, submit a new transaction with offline mode
    Offline(self::offline_mode::OfflineArgs),
}

impl Mode {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self {
            Self::Network(network_args) => {
                network_args
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
            Self::Offline(offline_args) => {
                offline_args
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
        }
    }
}

pub struct AddSubAccountCommandNetworkContext {
    pub connection_config: Option<crate::common::ConnectionConfig>,
}
