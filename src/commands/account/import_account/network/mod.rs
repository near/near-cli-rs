use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod print_keypair_to_terminal;
mod save_keypair_to_keychain;
mod save_keypair_to_legacy_keychain;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = NetworkForImportAccountContext)]
#[interactive_clap(output_context = NetworkForImportAccountOutputContext)]
// #[interactive_clap(output_context = NetworkForImportAccountArgsOutputContext)]
pub struct NetworkForImportAccount {
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    wallet_url: Option<crate::types::url::Url>,
    /// What is the name of the network?
    #[interactive_clap(skip_default_input_arg)]
    network_name: String,
    #[interactive_clap(subcommand)]
    /// How do you like to save imported account?
    save_mode: SaveMode,
}

pub type GetKeyStorePropertyAfterGettingNetworkCallback = std::sync::Arc<
    dyn Fn(
        &crate::config::NetworkConfig,
    ) -> color_eyre::eyre::Result<super::key_store_prop::KeyStorePropertyType>,
>;

#[derive(Clone)]
pub struct NetworkForImportAccountContext {
    pub config: crate::config::Config,
    pub account_id: near_primitives::types::AccountId,
    pub get_key_store_property_after_getting_network_callback:
        GetKeyStorePropertyAfterGettingNetworkCallback,
}

#[derive(Clone)]
pub struct NetworkForImportAccountOutputContext {
    pub config: crate::config::Config,
    pub chosen_network_config: crate::config::NetworkConfig,
    pub account_id: near_primitives::types::AccountId,
    pub key_store_property: super::key_store_prop::KeyStorePropertyType,
}

impl NetworkForImportAccountOutputContext {
    pub fn from_previous_context(
        previous_context: NetworkForImportAccountContext,
        scope: &<NetworkForImportAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let network_connection = previous_context.config.network_connection.clone();
        let mut network_config = network_connection
            .get(&scope.network_name)
            .expect("Failed to get network config!")
            .clone();
        if let Some(url) = scope.wallet_url.clone() {
            network_config.wallet_url = url.into();
        }

        let key_store_property = (previous_context
            .get_key_store_property_after_getting_network_callback)(
            &network_config
        )?;

        Ok(Self {
            config: previous_context.config,
            chosen_network_config: network_config,
            account_id: previous_context.account_id,
            key_store_property,
        })
    }
}

#[derive(Clone, Debug, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = NetworkForImportAccountOutputContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How to save an imported account:
pub enum SaveMode {
    #[strum_discriminants(strum(
        message = "save-to-keychain         - Save automatically generated key pair to keychain"
    ))]
    /// Save automatically generated key pair to keychain
    SaveToKeychain(save_keypair_to_keychain::SaveKeypairToKeychain),
    #[strum_discriminants(strum(
        message = "save-to-legacy-keychain  - Save automatically generated key pair to the legacy keychain (compatible with JS CLI)"
    ))]
    /// Save automatically generated key pair to the legacy keychain (compatible with JS CLI)
    SaveToLegacyKeychain(save_keypair_to_legacy_keychain::SaveKeypairToLegacyKeychain),
    #[strum_discriminants(strum(
        message = "print-to-terminal        - Print automatically generated key pair in terminal"
    ))]
    /// Print automatically generated key pair in terminal
    PrintToTerminal(print_keypair_to_terminal::PrintKeypairToTerminal),
}

impl NetworkForImportAccount {
    fn input_network_name(
        context: &NetworkForImportAccountContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(&context.config, &vec![context.account_id.clone()])
    }
}
