#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::SaveWithUseAutoGenerationContext)]
#[interactive_clap(output_context = SaveToKeychainContext)]
pub struct SaveToKeychain {
    #[interactive_clap(named_arg)]
    /// Select the network whose keychain will store this account
    network_config: crate::network::Network,
}

#[derive(Clone)]
struct SaveToKeychainContext(crate::network::NetworkContext);

impl SaveToKeychainContext {
    fn from_previous_context(
        previous_context: super::SaveWithUseAutoGenerationContext,
        _scope: &<SaveToKeychain as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let config = previous_context.global_context.config;
        let credentials_home_dir = config.credentials_home_dir.clone();
        Ok(Self(crate::network::NetworkContext {
            config,
            // An unfunded implicit account has no on-chain network to discover.
            interacting_with_account_ids: vec![],
            on_after_getting_network_callback: std::sync::Arc::new(move |network_config| {
                // Fail before generating or storing a key if its public picker
                // metadata cannot even have a credentials directory.
                std::fs::create_dir_all(&credentials_home_dir)?;
                let key_pair = crate::common::generate_keypair()?;
                let public_key = key_pair.public_key_str.parse()?;
                let account_id = &key_pair.implicit_account_id;
                let credentials = serde_json::to_string(&key_pair)?;
                crate::common::save_access_key_to_keychain(
                    network_config.clone(),
                    &credentials,
                    &public_key,
                    account_id.as_str(),
                )
                .map_err(|_| color_eyre::eyre::eyre!(
                    "Failed to save the generated implicit account to the system keychain. No plaintext file was written. To explicitly export a new account instead, choose use-auto-generation save-to-folder <folder-path>."
                ))?;

                crate::common::update_used_account_list_as_signer(
                    &credentials_home_dir,
                    account_id,
                );
                if !crate::common::get_used_account_list(&credentials_home_dir)
                    .iter()
                    .any(|account| account.used_as_signer && &account.account_id == account_id)
                {
                    color_eyre::eyre::bail!(
                        "Implicit account <{account_id}> with public key <{public_key}> was saved to the system keychain for network <{}>, but could not be added to the account picker. Check permissions for accounts.json in {}. No plaintext key was written.",
                        network_config.network_name,
                        credentials_home_dir.display()
                    );
                }
                println!(
                    "Implicit account <{account_id}> with public key <{public_key}> saved to the system keychain for network <{}>. Fund this account to use it on chain.",
                    network_config.network_name
                );
                Ok(())
            }),
        }))
    }
}

impl From<SaveToKeychainContext> for crate::network::NetworkContext {
    fn from(context: SaveToKeychainContext) -> Self {
        context.0
    }
}
