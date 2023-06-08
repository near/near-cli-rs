#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = LoginFromWebWalletContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct LoginFromWebWallet {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    wallet_url: Option<crate::types::url::Url>,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}

#[derive(Clone)]
pub struct LoginFromWebWalletContext(crate::network::NetworkContext);

impl LoginFromWebWalletContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<LoginFromWebWallet as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let config = previous_context.config.clone();
        let optional_wallet_url = scope.wallet_url.clone();

        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                move |network_config| {
                    let key_pair_properties: crate::common::KeyPairProperties =
                        crate::common::generate_keypair()?;
                    let mut url: url::Url = if let Some(url) = optional_wallet_url.clone() {
                        url.0.join("login/")?
                    } else {
                        network_config.wallet_url.join("login/")?
                    };
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
            on_after_getting_network_callback,
        }))
    }
}

impl From<LoginFromWebWalletContext> for crate::network::NetworkContext {
    fn from(item: LoginFromWebWalletContext) -> Self {
        item.0
    }
}

impl interactive_clap::FromCli for LoginFromWebWallet {
    type FromCliContext = crate::GlobalContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<LoginFromWebWallet as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();

        if clap_variant.wallet_url.is_none() {
            clap_variant.wallet_url = match Self::input_wallet_url(&context) {
                Ok(wallet_url) => wallet_url,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let wallet_url = clap_variant.wallet_url.clone();

        let new_context_scope = InteractiveClapContextScopeForLoginFromWebWallet { wallet_url };
        let output_context = match LoginFromWebWalletContext::from_previous_context(
            context.clone(),
            &new_context_scope,
        ) {
            Ok(new_context) => new_context,
            Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
        };
        let context = output_context;
        let optional_field = match clap_variant.network_config.take() {
            Some(ClapNamedArgNetworkForLoginFromWebWallet::NetworkConfig(cli_arg)) => Some(cli_arg),
            None => None,
        };
        match <crate::network::Network as interactive_clap::FromCli>::from_cli(
            optional_field,
            context.into(),
        ) {
            interactive_clap::ResultFromCli::Ok(cli_field) => {
                clap_variant.network_config = Some(
                    ClapNamedArgNetworkForLoginFromWebWallet::NetworkConfig(cli_field),
                );
            }
            interactive_clap::ResultFromCli::Cancel(optional_cli_field) => {
                clap_variant.network_config =
                    optional_cli_field.map(ClapNamedArgNetworkForLoginFromWebWallet::NetworkConfig);
                return interactive_clap::ResultFromCli::Cancel(Some(clap_variant));
            }
            interactive_clap::ResultFromCli::Back => return interactive_clap::ResultFromCli::Back,
            interactive_clap::ResultFromCli::Err(optional_cli_field, err) => {
                clap_variant.network_config =
                    optional_cli_field.map(ClapNamedArgNetworkForLoginFromWebWallet::NetworkConfig);
                return interactive_clap::ResultFromCli::Err(Some(clap_variant), err);
            }
        };
        interactive_clap::ResultFromCli::Ok(clap_variant)
    }
}

impl LoginFromWebWallet {
    fn input_wallet_url(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::url::Url>> {
        Ok(None)
    }
}
