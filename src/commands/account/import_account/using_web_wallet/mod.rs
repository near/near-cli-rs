#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = LoginFromWebWalletContext)]
pub struct LoginFromWebWallet {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}

#[derive(Clone)]
pub struct LoginFromWebWalletContext(crate::network::NetworkContext);

impl LoginFromWebWalletContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        _scope: &<LoginFromWebWallet as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::rc::Rc::new({
                let config = previous_context.config.clone();

                move |network_config| {
                    let key_pair_properties: crate::common::KeyPairProperties =
                        crate::common::generate_keypair()?;
                    let mut url: url::Url = network_config.wallet_url.join("login/")?;
                    url.query_pairs_mut()
                        .append_pair("title", "NEAR CLI")
                        .append_pair("public_key", &key_pair_properties.public_key_str);
                    // Use `success_url` once capture mode is implemented
                    //.append_pair("success_url", "http://127.0.0.1:8080");
                    eprintln!(
                        "If your browser doesn't automatically open, please visit this URL:\n {}\n",
                        &url.as_str()
                    );
                    // url.open();
                    open::that(url.as_ref()).ok();

                    let key_pair_properties_buf = serde_json::to_string(&key_pair_properties)?;
                    let error_message = format!("\nIt is currently not possible to verify the account access key.\nYou may not be logged in to {} or you may have entered an incorrect account_id.\nYou have the option to reconfirm your account or save your access key information.\n", &url.as_str());
                    super::login(
                        network_config.clone(),
                        config.credentials_home_dir.clone(),
                        &key_pair_properties_buf,
                        &key_pair_properties.public_key_str,
                        &error_message,
                    )
                }
            });

        Ok(Self(crate::network::NetworkContext {
            config: previous_context.config,
            interacting_with_account_ids: Vec::new(),
            on_after_getting_network_callback,
        }))
    }
}

impl From<LoginFromWebWalletContext> for crate::network::NetworkContext {
    fn from(item: LoginFromWebWalletContext) -> Self {
        item.0
    }
}
