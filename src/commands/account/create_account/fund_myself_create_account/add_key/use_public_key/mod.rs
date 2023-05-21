#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::NewAccountContext)]
#[interactive_clap(output_context = AddPublicKeyActionContext)]
pub struct AddPublicKeyAction {
    /// Enter the public key for this account:
    public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(named_arg)]
    /// What is the signer account ID?
    sign_as: super::super::sign_as::SignerAccountId,
}

#[derive(Clone)]
pub struct AddPublicKeyActionContext(super::super::AccountPropertiesContext);

impl AddPublicKeyActionContext {
    pub fn from_previous_context(
        previous_context: super::super::NewAccountContext,
        scope: &<AddPublicKeyAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let account_properties = super::super::AccountProperties {
            new_account_id: previous_context.new_account_id,
            initial_balance: previous_context.initial_balance,
            public_key: scope.public_key.clone().into(),
        };

        Ok(Self(super::super::AccountPropertiesContext {
            config: previous_context.config,
            account_properties,
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
        }))
    }
}

impl From<AddPublicKeyActionContext> for super::super::AccountPropertiesContext {
    fn from(item: AddPublicKeyActionContext) -> Self {
        item.0
    }
}
