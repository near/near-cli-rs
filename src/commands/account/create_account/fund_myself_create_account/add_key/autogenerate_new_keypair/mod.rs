use std::str::FromStr;

use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::super::NewAccountContext)]
#[interactive_clap(output_context = GenerateKeypairContext)]
pub struct GenerateKeypair {
    #[interactive_clap(subcommand)]
    save_mode: SaveMode,
}

#[derive(Debug, Clone)]
pub struct GenerateKeypairContext {
    global_context: crate::GlobalContext,
    account_properties: super::super::AccountProperties,
    key_pair_properties: crate::common::KeyPairProperties,
}

impl GenerateKeypairContext {
    pub fn from_previous_context(
        previous_context: super::super::NewAccountContext,
        _scope: &<GenerateKeypair as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let key_pair_properties: crate::common::KeyPairProperties =
            crate::common::generate_keypair()?;
        let public_key = near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?;
        let account_properties = super::super::AccountProperties {
            new_account_id: previous_context.new_account_id,
            initial_balance: previous_context.initial_balance,
            public_key,
        };

        Ok(Self {
            global_context: previous_context.global_context,
            account_properties,
            key_pair_properties,
        })
    }
}

impl From<GenerateKeypairContext> for super::super::AccountPropertiesContext {
    fn from(item: GenerateKeypairContext) -> Self {
        Self {
            global_context: item.global_context,
            account_properties: item.account_properties,
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
        }
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = GenerateKeypairContext)]
#[interactive_clap(output_context = SaveModeContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Save an access key for this account:
pub enum SaveMode {
    #[cfg(target_os = "macos")]
    #[strum_discriminants(strum(
        message = "save-to-macos-keychain   - Save automatically generated key pair to macOS keychain"
    ))]
    /// Save automatically generated key pair to macOS keychain
    SaveToMacosKeychain(SignAs),
    #[strum_discriminants(strum(
        message = "save-to-keychain         - Save automatically generated key pair to the legacy keychain (compatible with JS CLI)"
    ))]
    /// Save automatically generated key pair to the legacy keychain (compatible with JS CLI)
    SaveToKeychain(SignAs),
    #[strum_discriminants(strum(
        message = "print-to-terminal        - Print automatically generated key pair in terminal"
    ))]
    /// Print automatically generated key pair in terminal
    PrintToTerminal(SignAs),
}

#[derive(Clone)]
pub struct SaveModeContext(super::super::AccountPropertiesContext);

impl SaveModeContext {
    pub fn from_previous_context(
        previous_context: GenerateKeypairContext,
        scope: &<SaveMode as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let scope = *scope;

        let on_before_sending_transaction_callback: crate::transaction_signature_options::OnBeforeSendingTransactionCallback =
            std::sync::Arc::new({
                let new_account_id = previous_context.account_properties.new_account_id.clone();
                let key_pair_properties = previous_context.key_pair_properties.clone();
                let credentials_home_dir = previous_context.global_context.config.credentials_home_dir.clone();

                move |_signed_transaction, network_config, storage_message| {
                    match scope {
                        #[cfg(target_os = "macos")]
                        SaveModeDiscriminants::SaveToMacosKeychain => {
                            let key_pair_properties_buf =
                                serde_json::to_string(&key_pair_properties)?;
                            *storage_message = crate::common::save_access_key_to_macos_keychain(
                                network_config.clone(),
                                &key_pair_properties_buf,
                                &key_pair_properties.public_key_str,
                                new_account_id.as_ref(),
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
                                new_account_id.as_ref(),
                            )?;
                        }
                        SaveModeDiscriminants::PrintToTerminal => {
                            eprintln!("\n--------------------  Access key info for account <{}> ------------------\n", &new_account_id);
                            eprintln!(
                                "Master Seed Phrase: {}\nSeed Phrase HD Path: {}\nImplicit Account ID: {}\nPublic Key: {}\nSECRET KEYPAIR: {}",
                                key_pair_properties.master_seed_phrase,
                                key_pair_properties.seed_phrase_hd_path,
                                key_pair_properties.implicit_account_id,
                                key_pair_properties.public_key_str,
                                key_pair_properties.secret_keypair_str,
                            );
                            eprintln!("\n------------------------------------------------------------------------------------");
                        }
                    }
                    Ok(())
                }
            });

        Ok(Self(super::super::AccountPropertiesContext {
            global_context: previous_context.global_context,
            account_properties: previous_context.account_properties,
            on_before_sending_transaction_callback,
        }))
    }
}

impl From<SaveModeContext> for super::super::AccountPropertiesContext {
    fn from(item: SaveModeContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::super::AccountPropertiesContext)]
pub struct SignAs {
    #[interactive_clap(named_arg)]
    /// What is the signer account ID?
    sign_as: super::super::sign_as::SignerAccountId,
}
