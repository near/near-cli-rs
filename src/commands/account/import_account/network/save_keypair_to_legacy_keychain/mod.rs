#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::NetworkForImportAccountOutputContext)]
#[interactive_clap(output_context = SaveKeypairToLegacyKeychainContext)]
pub struct SaveKeypairToLegacyKeychain;

#[derive(Debug, Clone)]
pub struct SaveKeypairToLegacyKeychainContext;

impl SaveKeypairToLegacyKeychainContext {
    pub fn from_previous_context(
        previous_context: super::NetworkForImportAccountOutputContext,
        _scope: &<SaveKeypairToLegacyKeychain as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        println!(
            "{}",
            crate::common::save_access_key_to_legacy_keychain(
                previous_context.chosen_network_config,
                previous_context.config.credentials_home_dir,
                &serde_json::to_string(&previous_context.key_store_property)?,
                &previous_context.key_store_property.to_public_key_str(),
                previous_context.account_id.as_str()
            )?
        );

        Ok(Self)
    }
}
