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

        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                move |network_config| {
                    let mut account_key_pair: Option<
                        crate::transaction_signature_options::AccountKeyPair,
                    > = None;

                    #[cfg(target_os = "macos")]
                    {
                        account_key_pair =
                            super::using_web_wallet::account_key_pair_from_macos_keychain(
                                network_config,
                                &previous_context.account_id,
                            )?
                    };

                    if let Some(account_key_pair) = account_key_pair {
                        println!(
                            "Use private key <{}> to export <{}>.",
                            account_key_pair.private_key, previous_context.account_id
                        );
                    } else if let Some(account_key_pair) =
                        super::using_web_wallet::account_key_pair_from_keychain(
                            network_config,
                            &previous_context.account_id,
                            &config.credentials_home_dir,
                        )?
                    {
                        println!(
                            "Use private key <{}> to export <{}>.",
                            account_key_pair.private_key, previous_context.account_id
                        );
                    } else {
                        return Err(color_eyre::eyre::Report::msg(format!("The macOS keychain or keychain is missing access keys for the {} account.", previous_context.account_id)));
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

impl From<ExportAccountFromPrivateKeyContext> for crate::network::NetworkContext {
    fn from(item: ExportAccountFromPrivateKeyContext) -> Self {
        item.0
    }
}

#[derive(Debug, serde::Serialize)]
struct KeyPairProperties {
    public_key: near_crypto::PublicKey,
    private_key: near_crypto::SecretKey,
}
