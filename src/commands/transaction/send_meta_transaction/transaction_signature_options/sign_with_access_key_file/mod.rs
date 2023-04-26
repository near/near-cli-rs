use color_eyre::eyre::WrapErr;

use crate::common::JsonRpcClientExt;
use crate::common::RpcQueryResponseExt;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::super::network_for_transaction::NetworkForTransactionArgsContext)]
#[interactive_clap(output_context = SignAccessKeyFileContext)]
pub struct SignAccessKeyFile {
    /// What is the location of the account access key file (path/to/access-key-file.json)?
    file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(subcommand)]
    submit: super::Submit,
}

#[derive(Clone)]
pub struct SignAccessKeyFileContext(super::SubmitContext);

impl SignAccessKeyFileContext {
    pub fn from_previous_context(
        previous_context: super::super::network_for_transaction::NetworkForTransactionArgsContext,
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
                &previous_context.relayer_account_id,
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

        let actions = vec![near_primitives::transaction::Action::Delegate(
            previous_context.signed_delegate_action.clone(),
        )];

        let unsigned_transaction = near_primitives::transaction::Transaction {
            public_key: account_json.public_key,
            block_hash: rpc_query_response.block_hash,
            nonce: current_nonce + 1,
            signer_id: previous_context.relayer_account_id,
            receiver_id: previous_context
                .signed_delegate_action
                .delegate_action
                .sender_id,
            actions,
        };

        let signature = account_json
            .private_key
            .sign(unsigned_transaction.get_hash_and_size().0.as_ref());
        let signed_transaction = near_primitives::transaction::SignedTransaction::new(
            signature.clone(),
            unsigned_transaction,
        );

        eprintln!("\nYour transaction (delegate) was signed successfully.");

        Ok(Self(super::SubmitContext {
            network_config: previous_context.network_config,
            signed_transaction,
        }))
    }
}

impl From<SignAccessKeyFileContext> for super::SubmitContext {
    fn from(item: SignAccessKeyFileContext) -> Self {
        item.0
    }
}
