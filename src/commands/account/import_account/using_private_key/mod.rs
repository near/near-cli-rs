#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::ImportAccountCommandContext)]
#[interactive_clap(output_context = LoginFromPrivateKeyContext)]
pub struct LoginFromPrivateKey {
    /// Enter your private (secret) key:
    private_key: crate::types::secret_key::SecretKey,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: super::network::NetworkForImportAccount,
}

#[derive(Debug, Clone)]
pub struct LoginFromPrivateKeyContext {
    global_context: crate::GlobalContext,
    account_id: near_primitives::types::AccountId,
    key_store_property: super::key_store_prop::PrimitiveKey,
}

impl LoginFromPrivateKeyContext {
    pub fn from_previous_context(
        previous_context: super::ImportAccountCommandContext,
        scope: &<LoginFromPrivateKey as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let private_key: near_crypto::SecretKey = scope.private_key.clone().into();
        let public_key: near_crypto::PublicKey = private_key.public_key();
        let key_store_property = super::key_store_prop::PrimitiveKey {
            secret_key: private_key.clone(),
            public_key,
        };

        Ok(Self {
            global_context: previous_context.global_context,
            account_id: previous_context.account_id,
            key_store_property,
        })
    }
}

impl From<LoginFromPrivateKeyContext> for super::network::NetworkForImportAccountContext {
    fn from(item: LoginFromPrivateKeyContext) -> Self {
        let get_key_store_property_after_getting_network_callback: super::network::GetKeyStorePropertyAfterGettingNetworkCallback =
            std::sync::Arc::new({
                move |_network_config| {
                    let key_store_property = item.key_store_property.clone();

                    // TODO: prompt if user wants to check if account exists

                    Ok(super::key_store_prop::KeyStorePropertyType::Primitive(key_store_property))
                }
            });

        Self {
            config: item.global_context.config,
            account_id: item.account_id,
            get_key_store_property_after_getting_network_callback,
        }
    }
}
