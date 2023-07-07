#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = DeleteKeyCommandContext)]
pub struct DeleteKeyCommand {
    #[interactive_clap(skip_default_input_arg)]
    /// Which account should you delete the access key for?
    owner_account_id: crate::types::account_id::AccountId,
    /// Enter the public keys you wish to delete (separated by comma):
    public_keys: crate::types::public_key_list::PublicKeyList,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct DeleteKeyCommandContext {
    global_context: crate::GlobalContext,
    owner_account_id: near_primitives::types::AccountId,
    public_keys: Vec<near_crypto::PublicKey>,
}

impl DeleteKeyCommandContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<DeleteKeyCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            owner_account_id: scope.owner_account_id.clone().into(),
            public_keys: scope.public_keys.clone().into(),
        })
    }
}

impl From<DeleteKeyCommandContext> for crate::commands::ActionContext {
    fn from(item: DeleteKeyCommandContext) -> Self {
        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::sync::Arc::new(move |_network_config| {
                Ok(crate::commands::PrepopulatedTransaction {
                    signer_id: item.owner_account_id.clone(),
                    receiver_id: item.owner_account_id.clone(),
                    actions: item
                        .public_keys
                        .clone()
                        .into_iter()
                        .map(|public_key| {
                            near_primitives::transaction::Action::DeleteKey(
                                near_primitives::transaction::DeleteKeyAction { public_key },
                            )
                        })
                        .collect(),
                })
            });

        Self {
            global_context: item.global_context,
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

impl DeleteKeyCommand {
    pub fn input_owner_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        Ok(Some(
            crate::common::input_signer_account_id_from_used_account_list(
                &context.config.credentials_home_dir,
                "Which account should you delete the access key for?",
            )?,
        ))
    }
}
