#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::ExportAccountContext)]
#[interactive_clap(output_context = ExportAccountFromPrivateKeyContext)]
pub struct ExportAccountFromPrivateKey {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}

#[derive(Clone)]
pub struct ExportAccountFromPrivateKeyContext(crate::network::NetworkContext);

impl ExportAccountFromPrivateKeyContext {
    pub fn from_previous_context(
        previous_context: super::ExportAccountContext,
        _scope: &<ExportAccountFromPrivateKey as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let config = previous_context.global_context.config.clone();
        let account_id = previous_context.account_id.clone();

        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                move |network_config| {
                    #[cfg(target_os = "macos")]
                    {
                        if let Ok(account_key_pair) =
                            super::using_web_wallet::get_account_key_pair_from_macos_keychain(
                                network_config,
                                &account_id,
                            )
                        {
                            println!(
                                "Here is the private key for account <{}>: {}",
                                account_id, account_key_pair.private_key,
                            );
                            return Ok(());
                        }
                    }

                    if let Ok(account_key_pair) =
                        super::using_web_wallet::get_account_key_pair_from_keychain(
                            network_config,
                            &account_id,
                            &config.credentials_home_dir,
                        )
                    {
                        println!(
                            "Here is the private key for account <{}>: {}",
                            account_id, account_key_pair.private_key,
                        );
                    } else {
                        return Err(color_eyre::eyre::Report::msg(format!(
                            "There are no access keys in keychain to export for account <{}>.",
                            account_id
                        )));
                    };
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

impl From<ExportAccountFromPrivateKeyContext> for crate::network::NetworkContext {
    fn from(item: ExportAccountFromPrivateKeyContext) -> Self {
        item.0
    }
}