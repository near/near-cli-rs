use color_eyre::eyre::WrapErr;

use crate::common::JsonRpcClientExt;
use crate::common::RpcQueryResponseExt;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = SignAccessKeyFileContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignAccessKeyFile {
    /// What is the location of the account access key file (path/to/access-key-file.json)?
    file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub nonce: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    pub block_hash: Option<String>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    meta_transaction_valid_for: Option<u64>,
    #[interactive_clap(subcommand)]
    submit: super::Submit,
}

#[derive(Clone)]
pub struct SignAccessKeyFileContext {
    network_config: crate::config::NetworkConfig,
    signed_transaction_or_signed_delegate_action: super::SignedTransactionOrSignedDelegateAction,
    on_before_sending_transaction_callback:
        crate::transaction_signature_options::OnBeforeSendingTransactionCallback,
    on_after_sending_transaction_callback:
        crate::transaction_signature_options::OnAfterSendingTransactionCallback,
}

impl SignAccessKeyFileContext {
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignAccessKeyFile as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let network_config = previous_context.network_config.clone();

        let data =
            std::fs::read_to_string(&scope.file_path).wrap_err("Access key file not found!")?;
        let account_json: super::AccountKeyPair = serde_json::from_str(&data)
            .wrap_err_with(|| format!("Error reading data from file: {:?}", &scope.file_path))?;

        let rpc_query_response = network_config
            .json_rpc_client()
            .blocking_call_view_access_key(
                &previous_context.prepopulated_transaction.signer_id,
                &account_json.public_key,
                near_primitives::types::BlockReference::latest(),
            )
            .wrap_err(
                "Cannot sign a transaction due to an error while fetching the most recent nonce value",
            )?;

        let current_nonce = rpc_query_response
            .access_key_view()
            .wrap_err("Error current_nonce")?
            .nonce;

        let mut unsigned_transaction = near_primitives::transaction::Transaction {
            public_key: account_json.public_key.clone(),
            block_hash: rpc_query_response.block_hash,
            nonce: current_nonce + 1,
            signer_id: previous_context.prepopulated_transaction.signer_id,
            receiver_id: previous_context.prepopulated_transaction.receiver_id,
            actions: previous_context
                .prepopulated_transaction
                .actions
                .into_iter()
                .map(
                    |action_or_non_delegate_action| match action_or_non_delegate_action {
                        crate::commands::ActionOrNonDelegateAction::Action(action) => action,
                        crate::commands::ActionOrNonDelegateAction::NonDelegateAction(
                            non_delegate_action,
                        ) => near_primitives::transaction::Action::from(non_delegate_action),
                    },
                )
                .collect(),
        };

        (previous_context.on_before_signing_callback)(&mut unsigned_transaction, &network_config)?;

        let signature = account_json
            .private_key
            .sign(unsigned_transaction.get_hash_and_size().0.as_ref());

        if network_config.meta_transaction_relayer_url.is_some() {
            let max_block_height =
                rpc_query_response.block_height + scope.meta_transaction_valid_for.unwrap_or(1000);

            let signed_delegate_action = super::get_signed_delegate_action(
                unsigned_transaction,
                &account_json.public_key,
                account_json.private_key,
                max_block_height,
            );

            return Ok(Self {
                network_config: previous_context.network_config,
                signed_transaction_or_signed_delegate_action: signed_delegate_action.into(),
                on_before_sending_transaction_callback: previous_context
                    .on_before_sending_transaction_callback,
                on_after_sending_transaction_callback: previous_context
                    .on_after_sending_transaction_callback,
            });
        }

        let signed_transaction = near_primitives::transaction::SignedTransaction::new(
            signature.clone(),
            unsigned_transaction,
        );

        eprintln!("\nYour transaction was signed successfully.");
        eprintln!("Public key: {}", account_json.public_key);
        eprintln!("Signature: {}", signature);

        Ok(Self {
            network_config: previous_context.network_config,
            signed_transaction_or_signed_delegate_action: signed_transaction.into(),
            on_before_sending_transaction_callback: previous_context
                .on_before_sending_transaction_callback,
            on_after_sending_transaction_callback: previous_context
                .on_after_sending_transaction_callback,
        })
    }
}

impl From<SignAccessKeyFileContext> for super::SubmitContext {
    fn from(item: SignAccessKeyFileContext) -> Self {
        Self {
            network_config: item.network_config,
            signed_transaction_or_signed_delegate_action: item
                .signed_transaction_or_signed_delegate_action,
            on_before_sending_transaction_callback: item.on_before_sending_transaction_callback,
            on_after_sending_transaction_callback: item.on_after_sending_transaction_callback,
        }
    }
}

impl interactive_clap::FromCli for SignAccessKeyFile {
    type FromCliContext = crate::commands::TransactionContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<SignAccessKeyFile as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();

        if clap_variant.file_path.is_none() {
            clap_variant.file_path = match Self::input_file_path(&context) {
                Ok(Some(file_path)) => Some(file_path),
                Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let file_path = clap_variant.file_path.clone().expect("Unexpected error");
        if clap_variant.nonce.is_none() {
            clap_variant.nonce = match Self::input_nonce(&context) {
                Ok(optional_nonce) => optional_nonce,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let nonce = clap_variant.nonce;
        if clap_variant.block_hash.is_none() {
            clap_variant.block_hash = match Self::input_block_hash(&context) {
                Ok(optional_block_hash) => optional_block_hash,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let block_hash = clap_variant.block_hash.clone();
        if clap_variant.meta_transaction_valid_for.is_none() {
            clap_variant.meta_transaction_valid_for =
                match Self::input_meta_transaction_valid_for(&context) {
                    Ok(meta_transaction_valid_for) => meta_transaction_valid_for,
                    Err(err) => {
                        return interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
                    }
                };
        }
        let meta_transaction_valid_for = clap_variant.meta_transaction_valid_for;

        let new_context_scope = InteractiveClapContextScopeForSignAccessKeyFile {
            file_path,
            nonce,
            block_hash,
            meta_transaction_valid_for,
        };
        let output_context =
            match SignAccessKeyFileContext::from_previous_context(context, &new_context_scope) {
                Ok(new_context) => new_context,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };

        match super::Submit::from_cli(clap_variant.submit.take(), output_context.into()) {
            interactive_clap::ResultFromCli::Ok(submit) => {
                clap_variant.submit = Some(submit);
                interactive_clap::ResultFromCli::Ok(clap_variant)
            }
            interactive_clap::ResultFromCli::Cancel(optional_submit) => {
                clap_variant.submit = optional_submit;
                interactive_clap::ResultFromCli::Cancel(Some(clap_variant))
            }
            interactive_clap::ResultFromCli::Back => interactive_clap::ResultFromCli::Back,
            interactive_clap::ResultFromCli::Err(optional_submit, err) => {
                clap_variant.submit = optional_submit;
                interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
            }
        }
    }
}

impl SignAccessKeyFile {
    fn input_nonce(
        _context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<u64>> {
        Ok(None)
    }

    fn input_block_hash(
        _context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        Ok(None)
    }

    fn input_meta_transaction_valid_for(
        _context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<u64>> {
        Ok(Some(1000))
    }
}
