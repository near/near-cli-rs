use near_primitives::borsh::BorshSerialize;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::commands::TransactionContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignPrivateKey {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    pub signer_public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    pub signer_private_key: crate::types::secret_key::SecretKey,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    pub nonce: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    pub block_hash: Option<String>,
    #[interactive_clap(subcommand)]
    pub submit: Option<super::Submit>,
}

impl interactive_clap::FromCli for SignPrivateKey {
    type FromCliContext = crate::commands::TransactionContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<SignPrivateKey as interactive_clap::ToCli>::CliVariant>,
        _context: Self::FromCliContext,
    ) -> Result<Option<Self>, Self::FromCliError>
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let signer_public_key: crate::types::public_key::PublicKey = match optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.signer_public_key.clone())
        {
            Some(cli_public_key) => cli_public_key,
            None => super::input_signer_public_key()?,
        };
        let signer_private_key: crate::types::secret_key::SecretKey = match optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.signer_private_key.clone())
        {
            Some(signer_private_key) => signer_private_key,
            None => super::input_signer_private_key()?,
        };
        let nonce: Option<u64> = optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.nonce);
        let block_hash: Option<String> = optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.block_hash.clone());
        let submit: Option<super::Submit> =
            optional_clap_variant.and_then(|clap_variant| clap_variant.submit);
        Ok(Some(Self {
            signer_public_key,
            signer_private_key,
            nonce,
            block_hash,
            submit,
        }))
    }
}

impl SignPrivateKey {
    pub async fn process(
        &self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_config: crate::config::NetworkConfig,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        let signer_secret_key: near_crypto::SecretKey = self.signer_private_key.clone().into();
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
                // println!("\nUnsigned transaction:\n");
                // crate::common::print_transaction(prepopulated_unsigned_transaction.clone());
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
        let signature = signer_secret_key.sign(unsigned_transaction.get_hash_and_size().0.as_ref());
        let signed_transaction = near_primitives::transaction::SignedTransaction::new(
            signature.clone(),
            unsigned_transaction,
        );
        let serialize_to_base64 = near_primitives::serialize::to_base64(
            signed_transaction
                .try_to_vec()
                .expect("Transaction is not expected to fail on serialization"),
        );
        println!("\nYour transaction was signed successfully.");
        println!("Public key: {}", self.signer_public_key);
        println!("Signature: {}", signature);
        // crate::common::print_transaction(signed_transaction.transaction.clone());
        println!();
        match self.submit.clone() {
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
