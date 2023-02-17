use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::commands::TransactionContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignSeedPhrase {
    /// Enter the seed-phrase for this account
    master_seed_phrase: String,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(subcommand)]
    submit: super::Submit,
}

#[derive(Clone)]
pub struct SignSeedPhraseContext {
    network_config: crate::config::NetworkConfig,
    signed_transaction: near_primitives::transaction::SignedTransaction,
    on_after_sending_transaction_callback:
        crate::transaction_signature_options::OnAfterSendingTransactionCallback,
}

impl SignSeedPhraseContext {
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignSeedPhrase as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Result<Self, color_eyre::eyre::Error> {
        let network_config = previous_context.network_config.clone();

        let key_pair_properties = crate::common::get_key_pair_properties_from_seed_phrase(
            scope.seed_phrase_hd_path.clone(),
            scope.master_seed_phrase.clone(),
        )?;

        let signer_secret_key: near_crypto::SecretKey =
            near_crypto::SecretKey::from_str(&key_pair_properties.secret_keypair_str)?;
        let signer_public_key =
            near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?;
        let online_signer_access_key_response = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(network_config.json_rpc_client().call(
                near_jsonrpc_client::methods::query::RpcQueryRequest {
                    block_reference: near_primitives::types::Finality::Final.into(),
                    request: near_primitives::views::QueryRequest::ViewAccessKey {
                        account_id: previous_context.transaction.signer_id.clone(),
                        public_key: signer_public_key.clone(),
                    },
                },
            ))
            .map_err(|err| {
                println!("\nYour transaction was not successfully signed.\n");
                color_eyre::Report::msg(format!(
                    "Failed to fetch public key information for nonce: {:?}",
                    err
                ))
            })?;
        let current_nonce =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKey(
                online_signer_access_key,
            ) = online_signer_access_key_response.kind
            {
                online_signer_access_key.nonce
            } else {
                return Err(color_eyre::Report::msg("Error current_nonce".to_string()));
            };

        let mut unsigned_transaction = near_primitives::transaction::Transaction {
            public_key: signer_public_key.clone(),
            block_hash: online_signer_access_key_response.block_hash,
            nonce: current_nonce + 1,
            ..previous_context.transaction.clone()
        };

        (previous_context.on_before_signing_callback)(&mut unsigned_transaction, &network_config)?;

        let signature = signer_secret_key.sign(unsigned_transaction.get_hash_and_size().0.as_ref());
        let signed_transaction = near_primitives::transaction::SignedTransaction::new(
            signature.clone(),
            unsigned_transaction,
        );

        (previous_context.on_after_signing_callback)(&signed_transaction)?;

        for action in signed_transaction.transaction.actions.iter() {
            if let near_primitives::transaction::Action::FunctionCall(_) = action {
                println!("\nSigned transaction:\n");
                crate::common::print_transaction(signed_transaction.transaction.clone());
            }
        }

        println!("\nYour transaction was signed successfully.");
        println!("Public key: {}", signer_public_key);
        println!("Signature: {}", signature);

        Ok(Self {
            network_config: previous_context.network_config,
            signed_transaction,
            on_after_sending_transaction_callback: previous_context
                .on_after_sending_transaction_callback,
        })
    }
}

impl From<SignSeedPhraseContext> for super::SubmitContext {
    fn from(item: SignSeedPhraseContext) -> Self {
        Self {
            network_config: item.network_config,
            signed_transaction: item.signed_transaction,
            on_after_sending_transaction_callback: item.on_after_sending_transaction_callback,
        }
    }
}

impl interactive_clap::FromCli for SignSeedPhrase {
    type FromCliContext = crate::commands::TransactionContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<SignSeedPhrase as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> Result<Option<Self>, Self::FromCliError>
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let master_seed_phrase: String = match optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.master_seed_phrase.as_ref())
        {
            Some(master_seed_phrase) => master_seed_phrase.clone(),
            None => Self::input_master_seed_phrase(&context)?,
        };
        let seed_phrase_hd_path: crate::types::slip10::BIP32Path = match optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.seed_phrase_hd_path.as_ref())
        {
            Some(seed_phrase_hd_path) => seed_phrase_hd_path.clone(),
            None => Self::input_seed_phrase_hd_path(&context)?,
        };
        let new_context_scope = InteractiveClapContextScopeForSignSeedPhrase {
            seed_phrase_hd_path: seed_phrase_hd_path.clone(),
            master_seed_phrase: master_seed_phrase.clone(),
        };
        let seed_phrase_context =
            SignSeedPhraseContext::from_previous_context(context.clone(), &new_context_scope)?;
        let new_context = super::SubmitContext::from(seed_phrase_context.clone());

        let optional_submit = super::Submit::from_cli(
            optional_clap_variant.and_then(|clap_variant| clap_variant.submit),
            new_context,
        )?;
        let submit = if let Some(submit) = optional_submit {
            submit
        } else {
            return Ok(None);
        };
        Ok(Some(Self {
            master_seed_phrase,
            seed_phrase_hd_path,
            submit,
        }))
    }
}

impl SignSeedPhrase {
    fn input_seed_phrase_hd_path(
        _context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<crate::types::slip10::BIP32Path> {
        Ok(
            inquire::CustomType::new("Enter seed phrase HD Path [if not sure, keep the default]")
                .with_default(crate::types::slip10::BIP32Path::from_str("m/44'/397'/0'").unwrap())
                .prompt()?,
        )
    }
}
