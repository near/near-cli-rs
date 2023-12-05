#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = LoginFromPrivateKeyContext)]
pub struct LoginFromPrivateKey {
    /// Enter your private (secret) key:
    private_key: crate::types::secret_key::SecretKey,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}

#[derive(Clone)]
pub struct LoginFromPrivateKeyContext(crate::network::NetworkContext);

impl LoginFromPrivateKeyContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<LoginFromPrivateKey as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let private_key: near_crypto::SecretKey = scope.private_key.clone().into();
        let public_key = private_key.public_key();
        let key_pair_properties = KeyPairProperties {
            public_key: public_key.clone(),
            private_key,
        };
        let key_pair_properties_buf = serde_json::to_string(&key_pair_properties).unwrap();

        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let config = previous_context.config.clone();

                move |network_config| {
                    super::login(
                        network_config.clone(),
                        config.credentials_home_dir.clone(),
                        &key_pair_properties_buf,
                        &public_key.to_string(),
                        &format!("\nIt is currently not possible to verify the account access key on network <{}>.\nYou may have entered an incorrect account_id.\nYou have the option to reconfirm your account or save your access key information.\n",
                            network_config.network_name
                        )
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

impl From<LoginFromPrivateKeyContext> for crate::network::NetworkContext {
    fn from(item: LoginFromPrivateKeyContext) -> Self {
        item.0
    }
}

#[derive(Debug, serde::Serialize)]
struct KeyPairProperties {
    public_key: near_crypto::PublicKey,
    private_key: near_crypto::SecretKey,
}
