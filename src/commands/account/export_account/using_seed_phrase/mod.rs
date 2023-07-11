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
                        if let Ok(password_list) =
                            super::using_web_wallet::get_password_list_from_macos_keychain(
                                network_config,
                                &previous_context.account_id,
                            )
                        {
                            for password in password_list {
                                if let Ok(key_pair_properties) =
                                    serde_json::from_slice::<crate::common::KeyPairProperties>(
                                        &password,
                                    )
                                {
                                    println!(
                                        "Here is the secret recovery seed phrase for account <{}>: \"{}\" (HD Path: {}).",
                                        previous_context.account_id, key_pair_properties.master_seed_phrase, key_pair_properties.seed_phrase_hd_path
                                    );
                                    return Ok(());
                                }
                            }
                        }
                    }

                    let data_path = get_seed_phrase_data_path(
                        network_config,
                        &previous_context.account_id,
                        &config.credentials_home_dir,
                    )?;

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

pub fn get_seed_phrase_data_path(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
    credentials_home_dir: &std::path::Path,
) -> color_eyre::eyre::Result<std::path::PathBuf> {
    let check_if_seed_phrase_exists = true;
    super::using_web_wallet::get_account_properties_data_path(
        network_config,
        account_id,
        credentials_home_dir,
        check_if_seed_phrase_exists,
    )
}
