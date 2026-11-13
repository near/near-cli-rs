#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = LoginFromWebWalletContext)]
pub struct LoginFromWebWallet {
    #[interactive_clap(subargs)]
    details: super::ImportAccountDetails,
}

#[derive(Clone)]
pub struct LoginFromWebWalletContext {
    global_context: crate::GlobalContext,
    key_store_property: super::key_store_prop::RecoverableKey,
}

impl LoginFromWebWalletContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        _scope: &<LoginFromWebWallet as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            key_store_property: super::key_store_prop::RecoverableKey::generate_keypair()?,
        })
    }
}

impl From<LoginFromWebWalletContext> for super::ImportAccountDetailsContext {
    fn from(item: LoginFromWebWalletContext) -> Self {
        let key_store_property =
            super::key_store_prop::KeyStorePropertyType::Recoverable(item.key_store_property);

        let on_after_getting_network_callback: super::network::OnAfterGettingNetworkConfigCallback =
            std::sync::Arc::new({
                let public_key_str = key_store_property.to_public_key_str();

                move |network_config, will_check_account_id| {
                    let mut url: url::Url = network_config.wallet_url.join("login/")?;
                    url.query_pairs_mut()
                        .append_pair("title", "NEAR CLI")
                        .append_pair("public_key", &public_key_str);
                    // Use `success_url` once capture mode is implemented
                    //.append_pair("success_url", "http://127.0.0.1:8080");
                    eprintln!(
                        "If your browser doesn't automatically open, please visit this URL:\n {}\n",
                        url.as_str()
                    );
                    // url.open();
                    open::that(url.as_ref()).ok();

                    // Only the key check needs the key on-chain; saving it doesn't.
                    if will_check_account_id {
                        wait_for_web_wallet_approval()?;
                    }

                    Ok(())
                }
            });

        Self {
            global_context: item.global_context,
            key_store_property,
            on_after_getting_network_callback,
            uses_web_wallet: true,
        }
    }
}

/// Nothing tells the CLI when the wallet has added the key (there is no callback), so before the
/// `--check-account-id` step, in an interactive terminal, pause until the user says they've
/// approved it. Without this the check runs immediately and fails because the key isn't on-chain
/// yet. Non-interactive runs (scripts, CI) are not blocked.
fn wait_for_web_wallet_approval() -> color_eyre::eyre::Result<()> {
    use std::io::IsTerminal;

    if !std::io::stdin().is_terminal() {
        return Ok(());
    }

    tracing_indicatif::suspend_tracing_indicatif(|| {
        inquire::Text::new("Press Enter once you've approved adding the key in your wallet")
            .prompt()
    })?;

    Ok(())
}
