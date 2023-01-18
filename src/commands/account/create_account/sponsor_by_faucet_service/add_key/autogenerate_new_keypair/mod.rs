use std::str::FromStr;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

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
        account_properties: super::super::super::AccountProperties,
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
    SaveToMacosKeychain(SaveKeyPair),
    #[strum_discriminants(strum(
        message = "save-to-keychain         - Save automatically generated key pair to the legacy keychain (compatible with JS CLI)"
    ))]
    ///Save automatically generated key pair to the legacy keychain (compatible with JS CLI)
    SaveToKeychain(SaveKeyPair),
    #[strum_discriminants(strum(
        message = "print-to-terminal        - Print automatically generated key pair in terminal"
    ))]
    ///Print automatically generated key pair in terminal
    PrintToTerminal(SaveKeyPair),
}

impl SaveMode {
    pub fn get_network_config(
        &self,
        config: crate::config::Config,
    ) -> crate::config::NetworkConfig {
        match self {
            #[cfg(target_os = "macos")]
            Self::SaveToMacosKeychain(save_key_pair) => save_key_pair.get_network_config(config),
            Self::SaveToKeychain(save_key_pair) => save_key_pair.get_network_config(config),
            Self::PrintToTerminal(save_key_pair) => save_key_pair.get_network_config(config),
        }
    }

    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: super::super::super::AccountProperties,
    ) -> crate::CliResult {
        let key_pair_properties: crate::common::KeyPairProperties =
            crate::common::generate_keypair().await?;
        let public_key = near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?;
        let account_properties = super::super::super::AccountProperties {
            public_key,
            ..account_properties
        };
        let network_config = self.get_network_config(config.clone());
        match self {
            #[cfg(target_os = "macos")]
            SaveMode::SaveToMacosKeychain(save_key_pair) => {
                let key_pair_properties_buf = serde_json::to_string(&key_pair_properties)?;
                let storage_message = crate::common::save_access_key_to_macos_keychain(
                    network_config,
                    &key_pair_properties_buf,
                    &key_pair_properties.public_key_str,
                    &account_properties.new_account_id,
                )
                .map_err(|err| {
                    color_eyre::Report::msg(format!(
                        "Failed to save a file with access key: {}",
                        err
                    ))
                })?;
                save_key_pair
                    .process(config, account_properties, Some(storage_message))
                    .await
            }
            SaveMode::SaveToKeychain(save_key_pair) => {
                let key_pair_properties_buf = serde_json::to_string(&key_pair_properties)?;
                let storage_message = crate::common::save_access_key_to_keychain(
                    network_config,
                    config.credentials_home_dir.clone(),
                    &key_pair_properties_buf,
                    &key_pair_properties.public_key_str,
                    &account_properties.new_account_id,
                )
                .map_err(|err| {
                    color_eyre::Report::msg(format!(
                        "Failed to save a file with access key: {}",
                        err
                    ))
                })?;
                save_key_pair
                    .process(config, account_properties, Some(storage_message))
                    .await
            }
            SaveMode::PrintToTerminal(save_key_pair) => {
                let storage_message = format!(
                    "Master Seed Phrase: {}\nSeed Phrase HD Path: {}\nImplicit Account ID: {}\nPublic Key: {}\nSECRET KEYPAIR: {}",
                    key_pair_properties.master_seed_phrase,
                    key_pair_properties.seed_phrase_hd_path,
                    key_pair_properties.implicit_account_id,
                    key_pair_properties.public_key_str,
                    key_pair_properties.secret_keypair_str,
                );
                save_key_pair
                    .process(config, account_properties, Some(storage_message))
                    .await
            }
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SaveKeyPair {
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: super::super::network::Network,
}

impl SaveKeyPair {
    pub fn get_network_config(
        &self,
        config: crate::config::Config,
    ) -> crate::config::NetworkConfig {
        self.network_config.get_network_config(config)
    }

    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: super::super::super::AccountProperties,
        storage_message: Option<String>,
    ) -> crate::CliResult {
        self.network_config
            .process(config, account_properties, storage_message)
            .await
    }
}
