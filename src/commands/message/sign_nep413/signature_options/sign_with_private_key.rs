#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::FinalSignNep413Context)]
#[interactive_clap(output_context = SignPrivateKeyContext)]
pub struct SignPrivateKey {
    /// Enter your private (secret) key:
    pub private_key: crate::types::secret_key::SecretKey,
}

#[derive(Debug, Clone)]
pub struct SignPrivateKeyContext;

impl SignPrivateKeyContext {
    pub fn from_previous_context(
        previous_context: super::super::FinalSignNep413Context,
        scope: &<SignPrivateKey as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let secret_key: near_crypto::SecretKey = scope.private_key.clone().into();
        let public_key = secret_key.public_key();
        let signature = super::super::sign_nep413_payload(&previous_context.payload, &secret_key)?;

        let signed_message = super::super::SignedMessage {
            account_id: previous_context.signer_id.to_string(),
            public_key: public_key.to_string(),
            signature: signature.to_string(),
        };
        println!("{}", serde_json::to_string_pretty(&signed_message)?);
        Ok(Self)
    }
}
