#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::ImportAccountCommandContext)]
#[interactive_clap(output_context = LoginFromWebWalletContext)]
pub struct LoginFromWebWallet {
    #[interactive_clap(named_arg)]
    /// Select network
    network: super::network::NetworkForImportAccount,
}

#[derive(Clone)]
pub struct LoginFromWebWalletContext {
    global_context: crate::GlobalContext,
    account_id: near_primitives::types::AccountId,
    key_store_property: super::key_store_prop::RecoverableKey,
}

impl LoginFromWebWalletContext {
    pub fn from_previous_context(
        previous_context: super::ImportAccountCommandContext,
        _scope: &<LoginFromWebWallet as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            account_id: previous_context.account_id,
            key_store_property: super::key_store_prop::RecoverableKey::generate_keypair()?,
        })
    }
}

impl From<LoginFromWebWalletContext> for super::network::NetworkForImportAccountContext {
    fn from(item: LoginFromWebWalletContext) -> Self {
        let get_key_store_property_after_getting_network_callback: super::network::GetKeyStorePropertyAfterGettingNetworkCallback =
            std::sync::Arc::new({
                move |network_config| {
                    let key_store_property = super::key_store_prop::KeyStorePropertyType::Recoverable(item.key_store_property.clone());

                    let mut url: url::Url = network_config.wallet_url.join("login/")?;
                    url.query_pairs_mut()
                        .append_pair("title", "NEAR CLI")
                        .append_pair("public_key", &key_store_property.to_public_key_str());
                    // Use `success_url` once capture mode is implemented
                    //.append_pair("success_url", "http://127.0.0.1:8080");
                    eprintln!(
                        "If your browser doesn't automatically open, please visit this URL:\n {}\n",
                        url.as_str()
                    );
                    // url.open();
                    open::that(url.as_ref()).ok();

                    Ok(key_store_property)
                }
            });

        Self {
            global_context: item.global_context,
            account_id: item.account_id,
            get_key_store_property_after_getting_network_callback,
        }
    }
}
