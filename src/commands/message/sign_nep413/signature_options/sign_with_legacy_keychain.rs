#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::FinalSignNep413Context)]
#[interactive_clap(output_context = SignLegacyKeychainContext)]
pub struct SignLegacyKeychain {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}

pub struct SignLegacyKeychainContext(crate::network::NetworkContext);

impl SignLegacyKeychainContext {
    pub fn from_previous_context(
        previous_context: super::super::FinalSignNep413Context,
        _scope: &<SignLegacyKeychain as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let signer_id = previous_context.signer_id.clone();
                let payload = previous_context.payload.clone();
                let credentials_home_dir = previous_context
                    .global_context
                    .config
                    .credentials_home_dir
                    .clone();
                move |network_config| {
                    let key_pair = crate::commands::account::export_account::get_account_key_pair_from_legacy_keychain(
                        network_config,
                        &signer_id,
                        &credentials_home_dir
                    )?;
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

impl From<SignLegacyKeychainContext> for crate::network::NetworkContext {
    fn from(item: SignLegacyKeychainContext) -> Self {
        item.0
    }
}
