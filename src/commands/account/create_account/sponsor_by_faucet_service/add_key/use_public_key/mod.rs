#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::NewAccountContext)]
#[interactive_clap(output_context = AddPublicKeyActionContext)]
pub struct AddPublicKeyAction {
    /// Enter the public key for this account:
    public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: super::super::network::Network,
}

#[derive(Clone)]
pub struct AddPublicKeyActionContext(super::super::SponsorServiceContext);

impl AddPublicKeyActionContext {
    pub fn from_previous_context(
        previous_context: super::super::NewAccountContext,
        scope: &<AddPublicKeyAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(super::super::SponsorServiceContext {
            config: previous_context.config,
            new_account_id: previous_context.new_account_id,
            public_key: scope.public_key.clone().into(),
            on_after_getting_network_callback: std::sync::Arc::new(
                |_network_config, _storage_message| Ok(()),
            ),
            on_before_creating_account_callback: previous_context
                .on_before_creating_account_callback,
        }))
    }
}

impl From<AddPublicKeyActionContext> for super::super::SponsorServiceContext {
    fn from(item: AddPublicKeyActionContext) -> Self {
        item.0
    }
}
