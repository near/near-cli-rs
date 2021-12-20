use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod generate_keypair;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = ())]
pub struct ImplicitAccount {
    #[interactive_clap(subcommand)]
    pub public_key_mode: PublicKeyMode,
}

impl ImplicitAccount {
    pub async fn process(self) -> crate::CliResult {
        self.public_key_mode.process().await
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = ())]
///Select the mode for the public key
pub enum PublicKeyMode {
    #[strum_discriminants(strum(message = "Generate key pair"))]
    /// Generate key pair
    GenerateKeypair(self::generate_keypair::CliGenerateKeypair),
}

impl PublicKeyMode {
    pub async fn process(self) -> crate::CliResult {
        match self {
            PublicKeyMode::GenerateKeypair(cli_generate_keypair) => {
                cli_generate_keypair.process().await
            }
        }
    }
}
