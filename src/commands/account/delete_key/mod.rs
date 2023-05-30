mod use_publickeylist_type;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = DeleteKeyCommandContext)]
pub struct DeleteKeyCommand {
    /// Which account should you delete the access key for?
    owner_account_id: crate::types::account_id::AccountId,
    /// Enter the public keys you wish to delete (separated by comma):
    public_keys: self::use_publickeylist_type::PublicKeyList,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct DeleteKeyCommandContext {
    config: crate::config::Config,
    owner_account_id: near_primitives::types::AccountId,
    public_keys: Vec<near_crypto::PublicKey>,
}

impl DeleteKeyCommandContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<DeleteKeyCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.0,
            owner_account_id: scope.owner_account_id.clone().into(),
            public_keys: scope
                .public_keys
                .clone()
                .0
                .into_iter()
                .map(Into::into)
                .collect(),
        })
    }
}

impl From<DeleteKeyCommandContext> for crate::commands::ActionContext {
    fn from(item: DeleteKeyCommandContext) -> Self {
        let public_keys_clone = std::sync::Arc::new(item.public_keys.clone());
        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::sync::Arc::new(move |_network_config| {
                Ok(crate::commands::PrepopulatedTransaction {
                    signer_id: item.owner_account_id.clone(),
                    receiver_id: item.owner_account_id.clone(),
                    actions: public_keys_clone
                        .clone()
                        .iter()
                        .map(|public_key| {
                            near_primitives::transaction::Action::DeleteKey(
                                near_primitives::transaction::DeleteKeyAction {
                                    public_key: public_key.clone(),
                                },
                            )
                        })
                        .collect(),
                })
            });

        Self {
            config: item.config,
            on_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }
    }
}
