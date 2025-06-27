#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = PublicKeyFromLedgerContext)]
pub struct PublicKeyFromLedger {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
}

#[derive(Debug, Clone)]
pub struct PublicKeyFromLedgerContext {}

impl PublicKeyFromLedgerContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<PublicKeyFromLedger as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
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
        let verifying_key = near_ledger::get_public_key(seed_phrase_hd_path.into()).map_err(
            |near_ledger_error| {
                color_eyre::Report::msg(format!(
                    "An error occurred while trying to get PublicKey from Ledger device: {near_ledger_error:?}"
                ))
            },
        )?;
        let public_key = near_crypto::PublicKey::ED25519(near_crypto::ED25519PublicKey::from(
            verifying_key.to_bytes(),
        ));

        if let crate::Verbosity::Quiet = previous_context.verbosity {
            println!("{public_key}");
        } else {
            eprintln!("\nPublic key (printed to stdout): ");
            println!("{public_key}");
        }

        Ok(Self {})
    }
}

impl PublicKeyFromLedger {
    pub fn input_seed_phrase_hd_path(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
        crate::transaction_signature_options::sign_with_ledger::input_seed_phrase_hd_path()
    }
}
