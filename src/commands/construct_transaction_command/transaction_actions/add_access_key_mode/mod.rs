use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod add_access_key;
mod generate_keypair;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SenderContext)]
pub struct AddAccessKeyMode {
    #[interactive_clap(subcommand)]
    pub public_key_mode: PublicKeyMode,
}

impl AddAccessKeyMode {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        self.public_key_mode
            .process(prepopulated_unsigned_transaction, network_connection_config)
            .await
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = crate::common::SenderContext)]
///Select the mode for the public key
pub enum PublicKeyMode {
    #[strum_discriminants(strum(message = "Enter public key"))]
    /// Enter public key
    PublicKey(self::add_access_key::AddAccessKeyAction),
    #[strum_discriminants(strum(message = "Generate key pair"))]
    /// Generate key pair
    GenerateKeypair(self::generate_keypair::GenerateKeypair),
}

impl PublicKeyMode {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self {
            PublicKeyMode::PublicKey(add_access_key_action) => {
                add_access_key_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            PublicKeyMode::GenerateKeypair(cli_generate_keypair) => {
                cli_generate_keypair
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}
