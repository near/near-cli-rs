use std::str::FromStr;

use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::super::NewAccountContext)]
#[interactive_clap(output_context = GenerateKeypairContext)]
pub struct GenerateKeypair {
    #[interactive_clap(subcommand)]
    save_mode: SaveMode,
}

#[derive(Clone)]
pub struct GenerateKeypairContext {
    config: crate::config::Config,
    new_account_id: crate::types::account_id::AccountId,
    public_key: near_crypto::PublicKey,
    key_pair_properties: crate::common::KeyPairProperties,
    on_before_creating_account_callback: super::super::network::OnBeforeCreatingAccountCallback,
}

impl GenerateKeypairContext {
    pub fn from_previous_context(
        previous_context: super::super::NewAccountContext,
        _scope: &<GenerateKeypair as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let key_pair_properties: crate::common::KeyPairProperties =
            crate::common::generate_keypair()?;
        let public_key = near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?;

        Ok(Self {
            config: previous_context.config,
            new_account_id: previous_context.new_account_id,
            public_key,
            key_pair_properties,
            on_before_creating_account_callback: previous_context
                .on_before_creating_account_callback,
        })
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = GenerateKeypairContext)]
#[interactive_clap(output_context = SaveModeContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Save an access key for this account
pub enum SaveMode {
    #[cfg(target_os = "macos")]
    #[strum_discriminants(strum(
        message = "save-to-macos-keychain   - Save automatically generated key pair to macOS keychain"
    ))]
    /// Save automatically generated key pair to macOS keychain
    SaveToMacosKeychain(SaveKeyPair),
    #[strum_discriminants(strum(
        message = "save-to-keychain         - Save automatically generated key pair to the legacy keychain (compatible with JS CLI)"
    ))]
    /// Save automatically generated key pair to the legacy keychain (compatible with JS CLI)
    SaveToKeychain(SaveKeyPair),
    #[strum_discriminants(strum(
        message = "print-to-terminal        - Print automatically generated key pair in terminal"
    ))]
    /// Print automatically generated key pair in terminal
    PrintToTerminal(SaveKeyPair),
}

#[derive(Clone)]
pub struct SaveModeContext(super::super::SponsorServiceContext);

impl SaveModeContext {
    pub fn from_previous_context(
        previous_context: GenerateKeypairContext,
        scope: &<SaveMode as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let scope = scope.clone();

        let on_after_getting_network_callback: super::super::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let new_account_id_str = previous_context.new_account_id.to_string();
                let key_pair_properties = previous_context.key_pair_properties.clone();
                let credentials_home_dir = previous_context.config.credentials_home_dir.clone();

                move |network_config, storage_message| {
                    match scope {
                        #[cfg(target_os = "macos")]
                        SaveModeDiscriminants::SaveToMacosKeychain => {
                            let key_pair_properties_buf =
                                serde_json::to_string(&key_pair_properties)?;
                            *storage_message = crate::common::save_access_key_to_macos_keychain(
                                network_config.clone(),
                                &key_pair_properties_buf,
                                &key_pair_properties.public_key_str,
                                &new_account_id_str,
                            )?;
                        }
                        SaveModeDiscriminants::SaveToKeychain => {
                            let key_pair_properties_buf =
                                serde_json::to_string(&key_pair_properties)?;
                            *storage_message = crate::common::save_access_key_to_keychain(
                                network_config.clone(),
                                credentials_home_dir.clone(),
                                &key_pair_properties_buf,
                                &key_pair_properties.public_key_str,
                                &new_account_id_str,
                            )?;
                        }
                        SaveModeDiscriminants::PrintToTerminal => {
                            println!("\n--------------------  Access key info for account <{}> ------------------\n", &new_account_id_str);
                            println!(
                                "Master Seed Phrase: {}\nSeed Phrase HD Path: {}\nImplicit Account ID: {}\nPublic Key: {}\nSECRET KEYPAIR: {}",
                                key_pair_properties.master_seed_phrase,
                                key_pair_properties.seed_phrase_hd_path,
                                key_pair_properties.implicit_account_id,
                                key_pair_properties.public_key_str,
                                key_pair_properties.secret_keypair_str,
                            );
                            println!("\n------------------------------------------------------------------------------------");
                        }
                    }
                    Ok(())
                }
            });

        Ok(Self(super::super::SponsorServiceContext {
            config: previous_context.config,
            new_account_id: previous_context.new_account_id,
            public_key: previous_context.public_key,
            on_after_getting_network_callback,
            on_before_creating_account_callback: previous_context
                .on_before_creating_account_callback,
        }))
    }
}

impl From<SaveModeContext> for super::super::SponsorServiceContext {
    fn from(item: SaveModeContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::super::SponsorServiceContext)]
pub struct SaveKeyPair {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: super::super::network::Network,
}
