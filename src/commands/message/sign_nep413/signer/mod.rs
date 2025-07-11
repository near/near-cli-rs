#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = SignAsContext)]
#[interactive_clap(output_context = SignAsWrapperContext)]
pub struct SignAs {
    /// Which account to sign the message with:
    signer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    sign_with: super::signature_options::SignWith,
}

#[derive(Debug, Clone)]
pub struct SignAsContext {
    pub global_context: crate::GlobalContext,
    pub message: String,
    pub nonce: crate::types::nonce32_bytes::Nonce32,
    pub recipient: String,
}

#[derive(Debug, Clone)]
pub struct SignAsWrapperContext(super::FinalSignNep413Context);

impl SignAsWrapperContext {
    pub fn from_previous_context(
        previous_context: SignAsContext,
        scope: &<SignAs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let payload = super::NEP413Payload {
            message: previous_context.message,
            nonce: previous_context.nonce.as_array(),
            recipient: previous_context.recipient,
            callback_url: None,
        };

        Ok(Self(super::FinalSignNep413Context {
            global_context: previous_context.global_context,
            payload,
            signer_id: scope.signer_account_id.clone().into(),
        }))
    }
}

impl From<SignAsWrapperContext> for super::FinalSignNep413Context {
    fn from(item: SignAsWrapperContext) -> Self {
        item.0
    }
}
