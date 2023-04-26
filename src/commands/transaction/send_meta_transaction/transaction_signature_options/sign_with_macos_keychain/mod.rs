use near_primitives::borsh::BorshDeserialize;

use color_eyre::eyre::WrapErr;

use crate::common::JsonRpcClientExt;
use crate::common::RpcQueryResponseExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::network_for_transaction::NetworkForTransactionArgsContext)]
#[interactive_clap(output_context = SignMacosKeychainContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignMacosKeychain {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    nonce: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    block_hash: Option<String>,
    #[interactive_clap(subcommand)]
    submit: super::Submit,
}

#[derive(Clone)]
pub struct SignMacosKeychainContext(super::SubmitContext);

impl SignMacosKeychainContext {
    pub fn from_previous_context(
        previous_context: super::super::network_for_transaction::NetworkForTransactionArgsContext,
        _scope: &<SignMacosKeychain as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let network_config = previous_context.network_config.clone();

        let keychain = security_framework::os::macos::keychain::SecKeychain::default()
            .wrap_err("Failed to open keychain")?;

        let access_key_list = network_config
            .json_rpc_client()
            .blocking_call_view_access_key_list(
                &previous_context.relayer_account_id,
                near_primitives::types::Finality::Final.into(),
            )
            .wrap_err_with(|| {
                format!(
                    "Failed to fetch access key list for {}",
                    previous_context.relayer_account_id
                )
            })?
            .access_key_list_view()?;

        let service_name = std::borrow::Cow::Owned(format!(
            "near-{}-{}",
            network_config.network_name,
            previous_context.relayer_account_id.as_str()
        ));
        let password = access_key_list
            .keys
            .into_iter()
            .filter(|key| {
                matches!(
                    key.access_key.permission,
                    near_primitives::views::AccessKeyPermissionView::FullAccess
                )
            })
            .map(|key| key.public_key)
            .find_map(|public_key| {
                let (password, _) = keychain
                    .find_generic_password(
                        &service_name,
                        &format!("{}:{}", previous_context.relayer_account_id, public_key),
                    )
                    .ok()?;
                Some(password)
            })
            .ok_or_else(|| {
                color_eyre::eyre::eyre!(format!(
                    "There are no access keys for {} account in the macOS keychain.",
                    previous_context.relayer_account_id
                ))
            })?;

        let account_json: super::AccountKeyPair =
            serde_json::from_slice(password.as_ref()).wrap_err("Error reading data")?;

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

        let serialize_from_base64 =
            near_primitives::serialize::from_base64(&previous_context.transaction_hash).unwrap();

        let signed_delegate_action =
            near_primitives::delegate_action::SignedDelegateAction::try_from_slice(
                &serialize_from_base64,
            )?;

        let actions = vec![near_primitives::transaction::Action::Delegate(
            signed_delegate_action.clone(),
        )];

        let unsigned_transaction = near_primitives::transaction::Transaction {
            public_key: account_json.public_key,
            block_hash: rpc_query_response.block_hash,
            nonce: current_nonce + 1,
            signer_id: previous_context.relayer_account_id,
            receiver_id: signed_delegate_action.delegate_action.sender_id,
            actions,
        };

        let signature = account_json
            .private_key
            .sign(unsigned_transaction.get_hash_and_size().0.as_ref());
        let signed_transaction = near_primitives::transaction::SignedTransaction::new(
            signature.clone(),
            unsigned_transaction.clone(),
        );

        eprintln!("\nYour transaction (delegate) was signed successfully.");

        Ok(Self(super::SubmitContext {
            network_config: previous_context.network_config,
            signed_transaction,
        }))
    }
}

impl From<SignMacosKeychainContext> for super::SubmitContext {
    fn from(item: SignMacosKeychainContext) -> Self {
        item.0
    }
}

impl interactive_clap::FromCli for SignMacosKeychain {
    type FromCliContext = super::super::network_for_transaction::NetworkForTransactionArgsContext;
    type FromCliError = color_eyre::eyre::Error;
    fn from_cli(
        optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();

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

        let new_context_scope =
            InteractiveClapContextScopeForSignMacosKeychain { nonce, block_hash };
        let output_context =
            match SignMacosKeychainContext::from_previous_context(context, &new_context_scope) {
                Ok(new_context) => new_context,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };

        match super::Submit::from_cli(clap_variant.submit.take(), output_context.into()) {
            interactive_clap::ResultFromCli::Ok(cli_submit) => {
                clap_variant.submit = Some(cli_submit);
                interactive_clap::ResultFromCli::Ok(clap_variant)
            }
            interactive_clap::ResultFromCli::Cancel(optional_cli_submit) => {
                clap_variant.submit = optional_cli_submit;
                interactive_clap::ResultFromCli::Cancel(Some(clap_variant))
            }
            interactive_clap::ResultFromCli::Back => interactive_clap::ResultFromCli::Back,
            interactive_clap::ResultFromCli::Err(optional_cli_submit, err) => {
                clap_variant.submit = optional_cli_submit;
                interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
            }
        }
    }
}

impl SignMacosKeychain {
    pub fn input_nonce(
        _context: &super::super::network_for_transaction::NetworkForTransactionArgsContext,
    ) -> color_eyre::eyre::Result<Option<u64>> {
        Ok(None)
    }

    pub fn input_block_hash(
        _context: &super::super::network_for_transaction::NetworkForTransactionArgsContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        Ok(None)
    }
}
