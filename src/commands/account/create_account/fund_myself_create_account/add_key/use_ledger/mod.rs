#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::NewAccountContext)]
#[interactive_clap(output_context = AddAccessWithLedgerContext)]
pub struct AddAccessWithLedger {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(named_arg)]
    /// What is the signer account ID?
    sign_as: super::super::sign_as::SignerAccountId,
}

#[derive(Clone)]
pub struct AddAccessWithLedgerContext(super::super::AccountPropertiesContext);

impl AddAccessWithLedgerContext {
    pub fn from_previous_context(
        previous_context: super::super::NewAccountContext,
        scope: &<AddAccessWithLedger as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
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

        let account_properties = super::super::AccountProperties {
            new_account_id: previous_context.new_account_id,
            initial_balance: previous_context.initial_balance,
            public_key,
        };

        Ok(Self(super::super::AccountPropertiesContext {
            global_context: previous_context.global_context,
            account_properties,
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
        }))
    }
}

impl From<AddAccessWithLedgerContext> for super::super::AccountPropertiesContext {
    fn from(item: AddAccessWithLedgerContext) -> Self {
        item.0
    }
}

impl AddAccessWithLedger {
    pub fn input_seed_phrase_hd_path(
        _context: &super::super::NewAccountContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
        crate::transaction_signature_options::sign_with_ledger::input_seed_phrase_hd_path()
    }
}
