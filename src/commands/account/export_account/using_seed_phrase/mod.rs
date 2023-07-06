use color_eyre::eyre::WrapErr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::ExportAccountContext)]
#[interactive_clap(output_context = ExportAccountFromSeedPhraseContext)]
pub struct ExportAccountFromSeedPhrase {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}

#[derive(Clone)]
pub struct ExportAccountFromSeedPhraseContext(crate::network::NetworkContext);

impl ExportAccountFromSeedPhraseContext {
    pub fn from_previous_context(
        previous_context: super::ExportAccountContext,
        _scope: &<ExportAccountFromSeedPhrase as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let config = previous_context.global_context.config.clone();

        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                move |network_config| {
                    #[cfg(target_os = "macos")]
                    {
                        let password = super::using_web_wallet::get_password_from_macos_keychain(
                            network_config,
                            &previous_context.account_id,
                        )?;

                        if let Some(password) = password {
                            let key_pair_properties: crate::common::KeyPairProperties =
                                serde_json::from_slice(&password)?;
                            println!(
                                "Here is the secret recovery seed phrase for account <{}>: \"{}\" (HD Path: {}).",
                                previous_context.account_id, key_pair_properties.master_seed_phrase, key_pair_properties.seed_phrase_hd_path
                            );
                            return Ok(());
                        }
                    }

                    let data_path = super::using_web_wallet::get_account_properties_data_path(
                        network_config,
                        &previous_context.account_id,
                        &config.credentials_home_dir,
                        true,
                    )?;
                    if data_path.exists() {
                        let data = std::fs::read_to_string(&data_path)
                            .wrap_err("Access key file not found!")?;
                        let key_pair_properties: crate::common::KeyPairProperties =
                            serde_json::from_str(&data).wrap_err_with(|| {
                                format!("Error reading data from file: {:?}", &data_path)
                            })?;
                        println!(
                                "Here is the secret recovery seed phrase for account <{}>: \"{}\" (HD Path: {}).",
                                previous_context.account_id, key_pair_properties.master_seed_phrase, key_pair_properties.seed_phrase_hd_path
                            );
                    } else {
                        return Err(color_eyre::eyre::Report::msg(format!("There are no master seed phrase in keychain to export for account <{}>.", previous_context.account_id)));
                    };
                    Ok(())
                }
            });

        Ok(Self(crate::network::NetworkContext {
            config: previous_context.global_context.config,
            on_after_getting_network_callback,
        }))
    }
}

impl From<ExportAccountFromSeedPhraseContext> for crate::network::NetworkContext {
    fn from(item: ExportAccountFromSeedPhraseContext) -> Self {
        item.0
    }
}
