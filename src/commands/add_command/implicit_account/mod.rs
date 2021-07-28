use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod generate_keypair;

/// Generate key pair
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliImplicitAccount {
    #[clap(subcommand)]
    public_key_mode: Option<CliPublicKeyMode>,
}

#[derive(Debug, Clone)]
pub struct ImplicitAccount {
    pub public_key_mode: PublicKeyMode,
}

impl From<CliImplicitAccount> for ImplicitAccount {
    fn from(item: CliImplicitAccount) -> Self {
        let public_key_mode = match item.public_key_mode {
            Some(cli_public_key_mode) => PublicKeyMode::from(cli_public_key_mode),
            None => PublicKeyMode::choose_public_key_mode(),
        };
        Self { public_key_mode }
    }
}

impl ImplicitAccount {
    pub async fn process(self) -> crate::CliResult {
        self.public_key_mode.process().await
    }
}

#[derive(Debug, clap::Clap)]
pub enum CliPublicKeyMode {
    /// Generate key pair
    GenerateKeypair(self::generate_keypair::CliGenerateKeypair),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum PublicKeyMode {
    #[strum_discriminants(strum(message = "Generate key pair"))]
    GenerateKeypair(self::generate_keypair::CliGenerateKeypair),
}

impl From<CliPublicKeyMode> for PublicKeyMode {
    fn from(item: CliPublicKeyMode) -> Self {
        match item {
            CliPublicKeyMode::GenerateKeypair(cli_generate_keypair) => {
                PublicKeyMode::GenerateKeypair(cli_generate_keypair)
            }
        }
    }
}

impl PublicKeyMode {
    pub fn choose_public_key_mode() -> Self {
        let variants = PublicKeyModeDiscriminants::iter().collect::<Vec<_>>();
        let modes = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let select_mode = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a permission that you want to add to the access key:")
            .items(&modes)
            .default(0)
            .interact()
            .unwrap();
        match variants[select_mode] {
            PublicKeyModeDiscriminants::GenerateKeypair => {
                Self::from(CliPublicKeyMode::GenerateKeypair(Default::default()))
            }
        }
    }

    pub async fn process(self) -> crate::CliResult {
        match self {
            PublicKeyMode::GenerateKeypair(cli_generate_keypair) => {
                cli_generate_keypair.process().await
            }
        }
    }
}
