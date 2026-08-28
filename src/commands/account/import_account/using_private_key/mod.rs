#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = LoginFromPrivateKeyContext)]
pub struct LoginFromPrivateKey {
    /// Enter your private (secret) key:
    private_key: crate::types::secret_key::SecretKey,

    #[interactive_clap(subargs)]
    details: super::ImportAccountDetails,
}

#[derive(Debug, Clone)]
pub struct LoginFromPrivateKeyContext {
    global_context: crate::GlobalContext,
    key_store_property: super::key_store_prop::PrimitiveKey,
}

impl LoginFromPrivateKeyContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<LoginFromPrivateKey as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let secret_key: near_crypto::SecretKey = scope.private_key.clone().into();
        let public_key: near_crypto::PublicKey = secret_key.public_key();

        Ok(Self {
            global_context: previous_context,
            key_store_property: super::key_store_prop::PrimitiveKey {
                secret_key,
                public_key,
            },
        })
    }
}

impl From<LoginFromPrivateKeyContext> for super::ImportAccountDetailsContext {
    fn from(item: LoginFromPrivateKeyContext) -> Self {
        let key_store_property =
            super::key_store_prop::KeyStorePropertyType::Primitive(item.key_store_property);
        let on_after_getting_network_callback: super::network::OnAfterGettingNetworkConfigCallback =
            std::sync::Arc::new(move |_network_config| Ok(()));

        Self {
            global_context: item.global_context,
            key_store_property,
            on_after_getting_network_callback,
        }
    }
}
