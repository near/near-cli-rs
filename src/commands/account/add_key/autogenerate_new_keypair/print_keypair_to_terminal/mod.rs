#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::GenerateKeypairContext)]
#[interactive_clap(output_context = PrintKeypairToTerminalContext)]
pub struct PrintKeypairToTerminal {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct PrintKeypairToTerminalContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    permission: near_primitives::account::AccessKeyPermission,
    key_pair_properties: crate::common::KeyPairProperties,
    public_key: near_crypto::PublicKey,
}

impl PrintKeypairToTerminalContext {
    pub fn from_previous_context(
        previous_context: super::GenerateKeypairContext,
        _scope: &<PrintKeypairToTerminal as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            permission: previous_context.permission,
            key_pair_properties: previous_context.key_pair_properties,
            public_key: previous_context.public_key,
        })
    }
}

impl From<PrintKeypairToTerminalContext> for crate::commands::ActionContext {
    fn from(item: PrintKeypairToTerminalContext) -> Self {
        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let signer_account_id = item.signer_account_id.clone();

                move |_network_config| {
                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: signer_account_id.clone(),
                        receiver_id: signer_account_id.clone(),
                        actions: vec![near_primitives::transaction::Action::AddKey(Box::new(
                            near_primitives::transaction::AddKeyAction {
                                public_key: item.public_key.clone(),
                                access_key: near_primitives::account::AccessKey {
                                    nonce: 0,
                                    permission: item.permission.clone(),
                                },
                            },
                        ))],
                    })
                }
            });

        Self {
            global_context: item.global_context,
            interacting_with_account_ids: vec![item.signer_account_id],
            on_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                move |_outcome_view, _network_config| {
                    eprintln!("\n--------------------  Access key info ------------------\n");
                    eprintln!(
                        "Master Seed Phrase: {}\nSeed Phrase HD Path: {}\nImplicit Account ID: {}\nPublic Key: {}\nSECRET KEYPAIR: {}",
                        item.key_pair_properties.master_seed_phrase,
                        item.key_pair_properties.seed_phrase_hd_path,
                        item.key_pair_properties.implicit_account_id,
                        item.key_pair_properties.public_key_str,
                        item.key_pair_properties.secret_keypair_str,
                    );
                    eprintln!("\n--------------------------------------------------------");
                    Ok(())
                },
            ),
        }
    }
}
