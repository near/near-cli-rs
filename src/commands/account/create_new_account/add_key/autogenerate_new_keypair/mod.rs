use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod print_keypair_to_terminal;
mod save_keypair_to_keychain;
#[cfg(target_os = "macos")]
mod save_keypair_to_macos_keychain;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct GenerateKeypair {
    #[interactive_clap(subcommand)]
    save_mode: SaveMode,
}

impl GenerateKeypair {
    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: super::super::AccountProperties,
    ) -> crate::CliResult {
        self.save_mode.process(config, account_properties).await
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///Save an access key for this account
pub enum SaveMode {
    #[cfg(target_os = "macos")]
    #[strum_discriminants(strum(
        message = "save-to-macos-keychain   - Save automatically generated key pair to macOS keychain"
    ))]
    ///Save automatically generated key pair to macOS keychain
    SaveToMacosKeychain(self::save_keypair_to_macos_keychain::SaveKeypairToMacosKeychain),
    #[strum_discriminants(strum(
        message = "save-to-keychain         - Save automatically generated key pair to the legacy keychain (compatible with JS CLI)"
    ))]
    ///Save automatically generated key pair to the legacy keychain (compatible with JS CLI)
    SaveToKeychain(self::save_keypair_to_keychain::SaveKeypairToKeychain),
    #[strum_discriminants(strum(
        message = "print-to-terminal        - Print automatically generated key pair in terminal"
    ))]
    ///Print automatically generated key pair in terminal
    PrintToTerminal(self::print_keypair_to_terminal::PrintKeypairToTerminal),
}

impl SaveMode {
    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: super::super::AccountProperties,
    ) -> crate::CliResult {
        let key_pair_properties: crate::common::KeyPairProperties =
            crate::common::generate_keypair().await?;
        match self {
            #[cfg(target_os = "macos")]
            SaveMode::SaveToMacosKeychain(save_keypair_to_macos_keychain) => {
                save_keypair_to_macos_keychain
                    .process(config, key_pair_properties, account_properties)
                    .await
            }
            SaveMode::SaveToKeychain(save_keypair_to_keychain) => {
                save_keypair_to_keychain
                    .process(config, key_pair_properties, account_properties)
                    .await
            }
            SaveMode::PrintToTerminal(print_keypair_to_terminal) => {
                print_keypair_to_terminal
                    .process(config, key_pair_properties, account_properties)
                    .await
            }
        }
    }
}
