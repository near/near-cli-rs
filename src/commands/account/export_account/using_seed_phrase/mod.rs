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
        let account_id = previous_context.account_id.clone();

        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                move |network_config| {
                    if let Ok(password) =
                        super::get_password_from_keychain(network_config, &account_id)
                    {
                        if let Ok(key_pair_properties) =
                            serde_json::from_str::<crate::common::KeyPairProperties>(&password)
                        {
                            println!(
                                "Here is the secret recovery seed phrase for account <{}>: \"{}\" (HD Path: {}).",
                                account_id, key_pair_properties.master_seed_phrase, key_pair_properties.seed_phrase_hd_path
                            );
                            return Ok(());
                        }
                    }

                    let data_path = get_seed_phrase_data_path(
                        network_config,
                        &account_id,
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
                        account_id, key_pair_properties.master_seed_phrase, key_pair_properties.seed_phrase_hd_path
                    );
                    Ok(())
                }
            });

        Ok(Self(crate::network::NetworkContext {
            config: previous_context.global_context.config,
            interacting_with_account_ids: vec![previous_context.account_id],
            on_after_getting_network_callback,
        }))
    }
}

impl From<ExportAccountFromSeedPhraseContext> for crate::network::NetworkContext {
    fn from(item: ExportAccountFromSeedPhraseContext) -> Self {
        item.0
    }
}

fn get_seed_phrase_data_path(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
    credentials_home_dir: &std::path::Path,
) -> color_eyre::eyre::Result<std::path::PathBuf> {
    let check_if_seed_phrase_exists = true;
    super::get_account_properties_data_path(
        network_config,
        account_id,
        credentials_home_dir,
        check_if_seed_phrase_exists,
    )
}
