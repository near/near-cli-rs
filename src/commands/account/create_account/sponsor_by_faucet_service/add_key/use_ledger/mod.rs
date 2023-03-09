#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::NewAccountContext)]
#[interactive_clap(output_context = super::super::SponsorServiceContext)]
pub struct AddAccessWithLedger {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: super::super::network::Network,
}

#[derive(Clone)]
pub struct AddAccessWithLedgerContext(super::super::SponsorServiceContext);

impl AddAccessWithLedgerContext {
    pub fn from_previous_context(
        previous_context: super::super::NewAccountContext,
        _scope: &<AddAccessWithLedger as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let seed_phrase_hd_path = crate::transaction_signature_options::sign_with_ledger::SignLedger::input_seed_phrase_hd_path()?.unwrap();
        println!(
            "Please allow getting the PublicKey on Ledger device (HD Path: {})",
            seed_phrase_hd_path
        );
        let public_key = near_ledger::get_public_key(seed_phrase_hd_path.into()).map_err(
            |near_ledger_error| {
                color_eyre::Report::msg(format!(
                    "An error occurred while trying to get PublicKey from Ledger device: {:?}",
                    near_ledger_error
                ))
            },
        )?;
        let public_key = near_crypto::PublicKey::ED25519(near_crypto::ED25519PublicKey::from(
            public_key.to_bytes(),
        ));

        Ok(Self(super::super::SponsorServiceContext {
            config: previous_context.config,
            new_account_id: previous_context.new_account_id,
            public_key,
            on_after_getting_network_callback: std::sync::Arc::new(
                |_network_config, _storage_message| Ok(()),
            ),
            on_before_creating_account_callback: previous_context
                .on_before_creating_account_callback,
        }))
    }
}

impl From<AddAccessWithLedgerContext> for super::super::SponsorServiceContext {
    fn from(item: AddAccessWithLedgerContext) -> Self {
        item.0
    }
}
