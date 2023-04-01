use std::str::FromStr;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod print_keypair_to_terminal;
mod save_keypair_to_keychain;
#[cfg(target_os = "macos")]
mod save_keypair_to_macos_keychain;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::access_key_type::AccessTypeContext)]
#[interactive_clap(output_context = GenerateKeypairContext)]
pub struct GenerateKeypair {
    #[interactive_clap(subcommand)]
    save_mode: SaveMode,
}

#[derive(Debug, Clone)]
pub struct GenerateKeypairContext {
    config: crate::config::Config,
    signer_account_id: near_primitives::types::AccountId,
    permission: near_primitives::account::AccessKeyPermission,
    key_pair_properties: crate::common::KeyPairProperties,
    public_key: near_crypto::PublicKey,
}

impl GenerateKeypairContext {
    pub fn from_previous_context(
        previous_context: super::access_key_type::AccessTypeContext,
        _scope: &<GenerateKeypair as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let key_pair_properties: crate::common::KeyPairProperties =
            crate::common::generate_keypair()?;
        let public_key = near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?;
        Ok(Self {
            config: previous_context.config,
            signer_account_id: previous_context.signer_account_id,
            permission: previous_context.permission,
            key_pair_properties,
            public_key,
        })
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = GenerateKeypairContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Save an access key for this account
pub enum SaveMode {
    #[cfg(target_os = "macos")]
    #[strum_discriminants(strum(
        message = "save-to-macos-keychain   - Save automatically generated key pair to macOS keychain"
    ))]
    /// Save automatically generated key pair to macOS keychain
    SaveToMacosKeychain(self::save_keypair_to_macos_keychain::SaveKeypairToMacosKeychain),
    #[strum_discriminants(strum(
        message = "save-to-keychain         - Save automatically generated key pair to the legacy keychain (compatible with JS CLI)"
    ))]
    /// Save automatically generated key pair to the legacy keychain (compatible with JS CLI)
    SaveToKeychain(self::save_keypair_to_keychain::SaveKeypairToKeychain),
    #[strum_discriminants(strum(
        message = "print-to-terminal        - Print automatically generated key pair in terminal"
    ))]
    /// Print automatically generated key pair in terminal
    PrintToTerminal(self::print_keypair_to_terminal::PrintKeypairToTerminal),
}
