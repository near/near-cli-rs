use std::collections::HashMap;
use std::sync::Arc;

use color_eyre::eyre::WrapErr;
use inquire::{CustomType, Select};

use crate::common::{CallResultExt, JsonRpcClientExt};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::profile_args_type::ArgsContext)]
#[interactive_clap(output_context = SignerContext)]
pub struct Signer {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the signer account ID?
    signer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct SignerContext {
    pub global_context: crate::GlobalContext,
    pub account_id: near_primitives::types::AccountId,
    pub data: Vec<u8>,
    pub signer_account_id: near_primitives::types::AccountId,
}

impl SignerContext {
    pub fn from_previous_context(
        previous_context: super::profile_args_type::ArgsContext,
        scope: &<Signer as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            account_id: previous_context.account_id,
            data: previous_context.data,
            signer_account_id: scope.signer_account_id.clone().into(),
        })
    }
}

impl From<SignerContext> for crate::commands::ActionContext {
    fn from(item: SignerContext) -> Self {
        let account_id = item.account_id.clone();
        let signer_id = item.signer_account_id.clone();
        let data = item.data;

        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            Arc::new({
                move |network_config| {
                    get_prepopulated_transaction(network_config, &account_id, &signer_id, &data)
                }
            });

        let on_before_signing_callback: crate::commands::OnBeforeSigningCallback = Arc::new({
            let signer_account_id = item.signer_account_id.clone();
            let account_id = item.account_id.clone();
            move |prepopulated_unsigned_transaction, network_config| {
                let json_rpc_client = network_config.json_rpc_client();
                let public_key = prepopulated_unsigned_transaction.public_key().clone();
                let receiver_id = prepopulated_unsigned_transaction.receiver_id().clone();

                if let Some(near_primitives::transaction::Action::FunctionCall(action)) =
                    prepopulated_unsigned_transaction.actions_mut().get_mut(0)
                {
                    action.deposit = get_deposit(
                        &json_rpc_client,
                        &signer_account_id,
                        &public_key,
                        &account_id,
                        "profile",
                        &receiver_id,
                        near_token::NearToken::from_yoctonear(action.deposit),
                    )?
                    .as_yoctonear();
                    Ok(())
                } else {
                    color_eyre::eyre::bail!("Unexpected action to change components",);
                }
            }
        });

        let account_id = item.account_id.clone();

        let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = Arc::new({
            move |transaction_info, _network_config| {
                if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = transaction_info.status {
                    eprintln!("\nProfile for {account_id} updated successfully");
                } else {
                    color_eyre::eyre::bail!("Failed to update profile!");
                };
                Ok(())
            }
        });

        Self {
            global_context: item.global_context,
            interacting_with_account_ids: vec![item.account_id],
            get_prepopulated_transaction_after_getting_network_callback,
            on_before_signing_callback,
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
            on_after_sending_transaction_callback,
        }
    }
}

impl Signer {
    fn input_signer_account_id(
        context: &super::profile_args_type::ArgsContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        loop {
            let signer_account_id: crate::types::account_id::AccountId =
                CustomType::new("What is the signer account ID?")
                    .with_default(context.account_id.clone().into())
                    .prompt()?;
            if !crate::common::is_account_exist(
                &context.global_context.config.network_connection,
                signer_account_id.clone().into(),
            ) {
                eprintln!(
                    "\nThe account <{signer_account_id}> does not exist on [{}] networks.",
                    context.global_context.config.network_names().join(", ")
                );
                #[derive(strum_macros::Display)]
                enum ConfirmOptions {
                    #[strum(to_string = "Yes, I want to enter a new account name.")]
                    Yes,
                    #[strum(to_string = "No, I want to use this account name.")]
                    No,
                }
                let select_choose_input = Select::new(
                    "Do you want to enter another signer account id?",
                    vec![ConfirmOptions::Yes, ConfirmOptions::No],
                )
                .prompt()?;
                if let ConfirmOptions::No = select_choose_input {
                    return Ok(Some(signer_account_id));
                }
            } else {
                return Ok(Some(signer_account_id));
            }
        }
    }
}

#[tracing::instrument(
    name = "Creating a pre-populated transaction for signature ...",
    skip_all
)]
fn get_prepopulated_transaction(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
    signer_id: &near_primitives::types::AccountId,
    data: &[u8],
) -> color_eyre::eyre::Result<crate::commands::PrepopulatedTransaction> {
    let contract_account_id = network_config.get_near_social_account_id_from_network()?;
    let mut prepopulated_transaction = crate::commands::PrepopulatedTransaction {
        signer_id: signer_id.clone(),
        receiver_id: contract_account_id.clone(),
        actions: vec![],
    };

    let local_profile: serde_json::Value = serde_json::from_slice(data)?;
    let remote_profile =
        get_remote_profile(network_config, &contract_account_id, account_id.clone())?;

    let deposit = required_deposit(
        &network_config.json_rpc_client(),
        &contract_account_id,
        account_id,
        &local_profile,
        Some(&remote_profile),
    )?;

    let new_social_db_state = near_socialdb_client::types::socialdb_types::SocialDb {
        accounts: HashMap::from([(
            account_id.clone(),
            near_socialdb_client::types::socialdb_types::AccountProfile {
                profile: serde_json::from_value(local_profile)?,
            },
        )]),
    };

    let args = serde_json::to_string(&super::TransactionFunctionArgs {
        data: new_social_db_state,
    })?
    .into_bytes();

    prepopulated_transaction.actions = vec![near_primitives::transaction::Action::FunctionCall(
        Box::new(near_primitives::transaction::FunctionCallAction {
            method_name: "set".to_string(),
            args,
            gas: crate::common::NearGas::from_tgas(300).as_gas(),
            deposit: deposit.as_yoctonear(),
        }),
    )];

    Ok(prepopulated_transaction)
}

#[tracing::instrument(name = "Calculation of the required deposit ...", skip_all)]
fn required_deposit(
    json_rpc_client: &near_jsonrpc_client::JsonRpcClient,
    near_social_account_id: &near_primitives::types::AccountId,
    account_id: &near_primitives::types::AccountId,
    data: &serde_json::Value,
    prev_data: Option<&serde_json::Value>,
) -> color_eyre::eyre::Result<near_token::NearToken> {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(near_socialdb_client::required_deposit(
            json_rpc_client,
            near_social_account_id,
            account_id,
            data,
            prev_data,
        ))
}

#[tracing::instrument(name = "Update the required deposit ...", skip_all)]
fn get_deposit(
    json_rpc_client: &near_jsonrpc_client::JsonRpcClient,
    signer_account_id: &near_primitives::types::AccountId,
    signer_public_key: &near_crypto::PublicKey,
    account_id: &near_primitives::types::AccountId,
    key: &str,
    near_social_account_id: &near_primitives::types::AccountId,
    required_deposit: near_token::NearToken,
) -> color_eyre::eyre::Result<near_token::NearToken> {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(near_socialdb_client::get_deposit(
            json_rpc_client,
            signer_account_id,
            signer_public_key,
            account_id,
            key,
            near_social_account_id,
            required_deposit,
        ))
}

#[tracing::instrument(name = "Getting data about a remote profile ...", skip_all)]
fn get_remote_profile(
    network_config: &crate::config::NetworkConfig,
    near_social_account_id: &near_primitives::types::AccountId,
    account_id: near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<serde_json::Value> {
    match network_config
        .json_rpc_client()
        .blocking_call_view_function(
            near_social_account_id,
            "get",
            serde_json::to_vec(&serde_json::json!({
                "keys": vec![format!("{account_id}/profile/**")],
            }))?,
            near_primitives::types::Finality::Final.into(),
        )
        .wrap_err_with(|| {
            format!("Failed to fetch query for view method: 'get {account_id}/profile/**' (contract <{}> on network <{}>)",
                near_social_account_id,
                network_config.network_name
            )
        })?
        .parse_result_from_json::<near_socialdb_client::types::socialdb_types::SocialDb>()
        .wrap_err_with(|| {
            format!("Failed to parse view function call return value for {account_id}/profile.")
        })?
        .accounts
        .get(&account_id) {
            Some(account_profile) => Ok(serde_json::to_value(account_profile.profile.clone())?),
            None => Ok(serde_json::Value::Null)
        }
}
