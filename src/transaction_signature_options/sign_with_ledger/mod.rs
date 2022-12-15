use inquire::Text;
use near_primitives::borsh::BorshSerialize;
use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignLedger {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(skip)]
    signer_public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    nonce: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    block_hash: Option<String>,
    #[interactive_clap(subcommand)]
    submit: Option<super::Submit>,
}

impl interactive_clap::FromCli for SignLedger {
    type FromCliContext = crate::GlobalContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<SignLedger as interactive_clap::ToCli>::CliVariant>,
        _context: Self::FromCliContext,
    ) -> Result<Option<Self>, Self::FromCliError>
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let seed_phrase_hd_path = match optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.seed_phrase_hd_path.clone())
        {
            Some(hd_path) => hd_path,
            None => SignLedger::input_seed_phrase_hd_path(),
        };
        println!(
            "Please allow getting the PublicKey on Ledger device (HD Path: {})",
            seed_phrase_hd_path
        );
        let public_key = near_ledger::get_public_key(seed_phrase_hd_path.clone().into()).map_err(
            |near_ledger_error| {
                color_eyre::Report::msg(format!(
                    "An error occurred while trying to get PublicKey from Ledger device: {:?}",
                    near_ledger_error
                ))
            },
        )?;
        let signer_public_key: crate::types::public_key::PublicKey =
            near_crypto::PublicKey::ED25519(near_crypto::ED25519PublicKey::from(
                public_key.to_bytes(),
            ))
            .into();
        let submit: Option<super::Submit> =
            optional_clap_variant.and_then(|clap_variant| clap_variant.submit);
        Ok(Some(Self {
            seed_phrase_hd_path,
            signer_public_key,
            nonce: None,
            block_hash: None,
            submit,
        }))
    }
}

impl SignLedger {
    pub fn input_seed_phrase_hd_path() -> crate::types::slip10::BIP32Path {
        crate::types::slip10::BIP32Path::from_str(
            &Text::new("Enter seed phrase HD Path (if you not sure leave blank for default)")
                .with_initial_value("44'/397'/0'/0'/1'")
                .prompt()
                .unwrap(),
        )
        .unwrap()
    }

    pub async fn process(
        &self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_config: crate::config::NetworkConfig,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        let seed_phrase_hd_path = self.seed_phrase_hd_path.clone().into();
        let online_signer_access_key_response = network_config
            .json_rpc_client()
            .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::ViewAccessKey {
                    account_id: prepopulated_unsigned_transaction.signer_id.clone(),
                    public_key: self.signer_public_key.clone().into(),
                },
            })
            .await
            .map_err(|err| {
                println!("\nUnsigned transaction:\n");
                crate::common::print_transaction(prepopulated_unsigned_transaction.clone());
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
        let unsigned_transaction = near_primitives::transaction::Transaction {
            public_key: self.signer_public_key.clone().into(),
            block_hash: online_signer_access_key_response.block_hash,
            nonce: current_nonce + 1,
            ..prepopulated_unsigned_transaction
        };
        println!("\nUnsigned transaction:\n");
        crate::common::print_transaction(unsigned_transaction.clone());
        println!(
            "Confirm transaction signing on your Ledger device (HD Path: {})",
            seed_phrase_hd_path,
        );
        let signature = match near_ledger::sign_transaction(
            unsigned_transaction
                .try_to_vec()
                .expect("Transaction is not expected to fail on serialization"),
            seed_phrase_hd_path,
        ) {
            Ok(signature) => {
                near_crypto::Signature::from_parts(near_crypto::KeyType::ED25519, &signature)
                    .expect("Signature is not expected to fail on deserialization")
            }
            Err(near_ledger_error) => {
                return Err(color_eyre::Report::msg(format!(
                    "Error occurred while signing the transaction: {:?}",
                    near_ledger_error
                )));
            }
        };

        let signed_transaction =
            near_primitives::transaction::SignedTransaction::new(signature, unsigned_transaction);
        let serialize_to_base64 = near_primitives::serialize::to_base64(
            signed_transaction
                .try_to_vec()
                .expect("Transaction is not expected to fail on serialization"),
        );
        println!("Your transaction was signed successfully.");
        match &self.submit {
            None => {
                let submit = super::Submit::choose_submit();
                submit
                    .process(network_config, signed_transaction, serialize_to_base64)
                    .await
            }
            Some(submit) => {
                submit
                    .process(network_config, signed_transaction, serialize_to_base64)
                    .await
            }
        }
    }
}
