#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::ExportAccountContext)]
#[interactive_clap(output_context = ExportAccountFromWebWalletContext)]
pub struct ExportAccountFromWebWallet {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}

#[derive(Clone)]
pub struct ExportAccountFromWebWalletContext(crate::network::NetworkContext);

impl ExportAccountFromWebWalletContext {
    pub fn from_previous_context(
        previous_context: super::ExportAccountContext,
        _scope: &<ExportAccountFromWebWallet as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let config = previous_context.global_context.config.clone();
        let account_id = previous_context.account_id.clone();

        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::rc::Rc::new({
                move |network_config| {
                    if let Ok(account_key_pair) =
                        super::get_account_key_pair_from_keychain(network_config, &account_id)
                    {
                        return auto_import_secret_key(
                            network_config,
                            &account_id,
                            &account_key_pair.private_key,
                        );
                    }

                    let account_key_pair = super::get_account_key_pair_from_legacy_keychain(
                        network_config,
                        &account_id,
                        &config.credentials_home_dir,
                    )?;
                    auto_import_secret_key(
                        network_config,
                        &account_id,
                        &account_key_pair.private_key,
                    )
                }
            });

        Ok(Self(crate::network::NetworkContext {
            config: previous_context.global_context.config,
            interacting_with_account_ids: vec![previous_context.account_id],
            on_after_getting_network_callback,
        }))
    }
}

impl From<ExportAccountFromWebWalletContext> for crate::network::NetworkContext {
    fn from(item: ExportAccountFromWebWalletContext) -> Self {
        item.0
    }
}

fn auto_import_secret_key(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
    private_key: &near_crypto::SecretKey,
) -> crate::CliResult {
    let mut url: url::Url = network_config.wallet_url.join("auto-import-secret-key")?;
    let fragment = format!("{}/{}", account_id, private_key);
    url.set_fragment(Some(&fragment));
    eprintln!(
        "If your browser doesn't automatically open, please visit this URL:\n {}\n",
        &url.as_str()
    );
    // url.open();
    open::that(url.as_ref()).ok();
    Ok(())
}
