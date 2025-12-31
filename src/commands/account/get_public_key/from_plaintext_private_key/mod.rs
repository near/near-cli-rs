#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = PublicKeyFromPlaintextPrivateKeyContext)]
pub struct PublicKeyFromPlaintextPrivateKey {
    /// Enter your private (secret) key:
    private_key: crate::types::secret_key::SecretKey,
}

#[derive(Debug, Clone)]
pub struct PublicKeyFromPlaintextPrivateKeyContext {}

impl PublicKeyFromPlaintextPrivateKeyContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<PublicKeyFromPlaintextPrivateKey as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let private_key: near_crypto::SecretKey = scope.private_key.clone().into();
        let public_key = private_key.public_key();

        if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe =
            previous_context.verbosity
        {
            eprint!("Public key (printed to stdout): ");
        }
        println!("{public_key}");

        Ok(Self {})
    }
}
