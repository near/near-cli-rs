#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::access_key_type::AccessTypeContext)]
#[interactive_clap(output_context = AddLedgerKeyActionContext)]
pub struct AddLedgerKeyAction {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct AddLedgerKeyActionContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    permission: near_primitives::account::AccessKeyPermission,
    public_key: crate::types::public_key::PublicKey,
}

impl AddLedgerKeyActionContext {
    pub fn from_previous_context(
        previous_context: super::access_key_type::AccessTypeContext,
        scope: &<AddLedgerKeyAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let seed_phrase_hd_path = scope.seed_phrase_hd_path.clone();
        eprintln!("Opening the NEAR application... Please approve opening the application");
        near_ledger::open_near_application().map_err(|ledger_error| {
            color_eyre::Report::msg(format!("An error happened while trying to open the NEAR application on the ledger: {ledger_error:?}"))
        })?;

        std::thread::sleep(std::time::Duration::from_secs(1));

        eprintln!(
            "Please allow getting the PublicKey on Ledger device (HD Path: {seed_phrase_hd_path})"
        );
        let public_key = near_ledger::get_public_key(seed_phrase_hd_path.into()).map_err(
            |near_ledger_error| {
                color_eyre::Report::msg(format!(
                    "An error occurred while trying to get PublicKey from Ledger device: {near_ledger_error:?}"
                ))
            },
        )?;
        let public_key = near_crypto::PublicKey::ED25519(near_crypto::ED25519PublicKey::from(
            public_key.to_bytes(),
        ));

        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            permission: previous_context.permission,
            public_key: public_key.into(),
        })
    }
}

impl From<AddLedgerKeyActionContext> for crate::commands::ActionContext {
    fn from(item: AddLedgerKeyActionContext) -> Self {
        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let signer_account_id = item.signer_account_id.clone();

                move |_network_config| {
                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: signer_account_id.clone(),
                        receiver_id: signer_account_id.clone(),
                        actions: vec![near_primitives::transaction::Action::AddKey(Box::new(
                            near_primitives::transaction::AddKeyAction {
                                public_key: item.public_key.clone().into(),
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
            get_prepopulated_transaction_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }
    }
}

impl AddLedgerKeyAction {
    pub fn input_seed_phrase_hd_path(
        _context: &super::access_key_type::AccessTypeContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
        crate::transaction_signature_options::sign_with_ledger::input_seed_phrase_hd_path()
    }
}
