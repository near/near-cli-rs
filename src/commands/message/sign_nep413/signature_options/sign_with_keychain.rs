#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::FinalSignNep413Context)]
#[interactive_clap(output_context = SignKeychainContext)]
pub struct SignKeychain {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}

pub struct SignKeychainContext(crate::network::NetworkContext);

impl SignKeychainContext {
    pub fn from_previous_context(
        previous_context: super::super::FinalSignNep413Context,
        _scope: &<SignKeychain as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let signer_id = previous_context.signer_id.clone();
                let payload = previous_context.payload.clone();
                move |network_config| {
                    let key_pair = crate::commands::account::export_account::get_account_key_pair_from_keychain(network_config, &signer_id)?;
                    let signature =
                        super::super::sign_nep413_payload(&payload, &key_pair.private_key)?;

                    let signed_message = super::super::SignedMessage {
                        account_id: signer_id.to_string(),
                        public_key: key_pair.public_key.to_string(),
                        signature: signature.to_string(),
                    };
                    println!("{}", serde_json::to_string_pretty(&signed_message)?);
                    Ok(())
                }
            });

        Ok(Self(crate::network::NetworkContext {
            config: previous_context.global_context.config,
            interacting_with_account_ids: vec![previous_context.signer_id],
            on_after_getting_network_callback,
        }))
    }
}

impl From<SignKeychainContext> for crate::network::NetworkContext {
    fn from(item: SignKeychainContext) -> Self {
        item.0
    }
}
