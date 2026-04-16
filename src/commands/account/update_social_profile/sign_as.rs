use std::collections::HashMap;
use std::sync::Arc;

use color_eyre::{eyre::WrapErr, owo_colors::OwoColorize};
use inquire::{CustomType, Select};

use crate::common::{
    CallResultExt, blocking_view_access_key, blocking_view_function,
};

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
    pub account_id: near_kit::AccountId,
    pub data: Vec<u8>,
    pub signer_account_id: near_kit::AccountId,
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
                let public_key: near_kit::PublicKey = prepopulated_unsigned_transaction
                    .public_key
                    .to_string()
                    .parse()
                    .wrap_err("Failed to convert public key")?;
                let receiver_id = prepopulated_unsigned_transaction.receiver_id.clone();

                if let Some(near_kit::Action::FunctionCall(action)) =
                    prepopulated_unsigned_transaction.actions.first_mut()
                {
                    action.deposit = get_deposit(
                        network_config,
                        &signer_account_id,
                        &public_key,
                        &account_id,
                        "profile",
                        &receiver_id,
                        action.deposit,
                    )?;
                    Ok(())
                } else {
                    color_eyre::eyre::bail!("Unexpected action to change components",);
                }
            }
        });

        let account_id = item.account_id.clone();
        let verbosity = item.global_context.verbosity;

        let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = Arc::new({
            move |transaction_info, _network_config| {
                if transaction_info.is_success() {
                    if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe = verbosity {
                        eprintln!("Profile for {account_id} updated successfully");
                    }
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
            sign_as_delegate_action: false,
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
                &context.global_context,
                signer_account_id.clone().into(),
            )? {
                tracing::warn!(
                    "{}",
                    format!(
                        "The account <{signer_account_id}> does not exist on [{}] networks.",
                        context.global_context.config.network_names().join(", ")
                    )
                    .red()
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
    account_id: &near_kit::AccountId,
    signer_id: &near_kit::AccountId,
    data: &[u8],
) -> color_eyre::eyre::Result<crate::commands::PrepopulatedTransaction> {
    tracing::info!(target: "near_teach_me", "Creating a pre-populated transaction for signature ...");
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
        network_config,
        &contract_account_id,
        account_id,
        &local_profile,
        Some(&remote_profile),
    )?;

    let new_social_db_state = crate::types::socialdb::SocialDb {
        accounts: HashMap::from([(
            account_id.clone(),
            crate::types::socialdb::AccountProfile {
                profile: serde_json::from_value(local_profile)?,
            },
        )]),
    };

    let args = serde_json::to_string(&super::TransactionFunctionArgs {
        data: new_social_db_state,
    })?
    .into_bytes();

    prepopulated_transaction.actions = vec![near_kit::Action::FunctionCall(
        near_kit::FunctionCallAction {
            method_name: "set".to_string(),
            args,
            gas: near_kit::Gas::from_tgas(300),
            deposit,
        },
    )];

    Ok(prepopulated_transaction)
}

#[tracing::instrument(name = "Calculation of the required deposit ...", skip_all)]
fn required_deposit(
    network_config: &crate::config::NetworkConfig,
    near_social_account_id: &near_kit::AccountId,
    account_id: &near_kit::AccountId,
    data: &serde_json::Value,
    prev_data: Option<&serde_json::Value>,
) -> color_eyre::eyre::Result<near_token::NearToken> {
    tracing::info!(target: "near_teach_me", "Calculation of the required deposit ...");

    const STORAGE_COST_PER_BYTE: i128 = 10i128.pow(19);
    const MIN_STORAGE_BALANCE: u128 = STORAGE_COST_PER_BYTE as u128 * 2000;
    const INITIAL_ACCOUNT_STORAGE_BALANCE: i128 = STORAGE_COST_PER_BYTE * 500;
    const EXTRA_STORAGE_BALANCE: i128 = STORAGE_COST_PER_BYTE * 5000;

    let storage_balance_result: color_eyre::eyre::Result<crate::types::socialdb::StorageBalance> = {
        let result = blocking_view_function(
            network_config,
            near_social_account_id,
            "storage_balance_of",
            serde_json::json!({ "account_id": account_id })
                .to_string()
                .into_bytes(),
            near_kit::Finality::Final.into(),
        )
        .wrap_err("Failed to fetch query for view method: 'storage_balance_of'")?;
        result
            .parse_result_from_json::<crate::types::socialdb::StorageBalance>()
    };

    let (available_storage, initial_account_storage_balance, min_storage_balance) =
        if let Ok(storage_balance) = storage_balance_result {
            (storage_balance.available, 0, 0)
        } else {
            (0, INITIAL_ACCOUNT_STORAGE_BALANCE, MIN_STORAGE_BALANCE)
        };

    let estimated_storage_balance = u128::try_from(
        STORAGE_COST_PER_BYTE * estimate_data_size(data, prev_data) as i128
            + initial_account_storage_balance
            + EXTRA_STORAGE_BALANCE,
    )
    .unwrap_or(0)
    .saturating_sub(available_storage);
    Ok(near_token::NearToken::from_yoctonear(std::cmp::max(
        estimated_storage_balance,
        min_storage_balance,
    )))
}

/// https://github.com/NearSocial/VM/blob/24055641b53e7eeadf6efdb9c073f85f02463798/src/lib/data/utils.js#L182-L198
fn estimate_data_size(data: &serde_json::Value, prev_data: Option<&serde_json::Value>) -> isize {
    const ESTIMATED_KEY_VALUE_SIZE: isize = 40 * 3 + 8 + 12;
    const ESTIMATED_NODE_SIZE: isize = 40 * 2 + 8 + 10;

    match data {
        serde_json::Value::Object(data) => {
            let inner_data_size = data
                .iter()
                .map(|(key, value)| {
                    let prev_value = if let Some(serde_json::Value::Object(prev_data)) = prev_data {
                        prev_data.get(key)
                    } else {
                        None
                    };
                    if prev_value.is_some() {
                        estimate_data_size(value, prev_value)
                    } else {
                        key.len() as isize * 2
                            + estimate_data_size(value, None)
                            + ESTIMATED_KEY_VALUE_SIZE
                    }
                })
                .sum();
            if prev_data.map(serde_json::Value::is_object).unwrap_or(false) {
                inner_data_size
            } else {
                ESTIMATED_NODE_SIZE + inner_data_size
            }
        }
        serde_json::Value::String(data) => {
            data.len().max(8) as isize
                - prev_data
                    .and_then(serde_json::Value::as_str)
                    .map(str::len)
                    .unwrap_or(0) as isize
        }
        _ => {
            unreachable!("estimate_data_size expects only Object or String values");
        }
    }
}

fn is_signer_access_key_function_call_access_can_call_set_on_social_db_account(
    near_social_account_id: &near_kit::AccountId,
    access_key_permission: &near_kit::AccessKeyPermissionView,
) -> color_eyre::eyre::Result<bool> {
    if let near_kit::AccessKeyPermissionView::FunctionCall {
        allowance: _,
        receiver_id,
        method_names,
    } = access_key_permission
    {
        Ok(receiver_id == near_social_account_id
            && (method_names.contains(&"set".to_string()) || method_names.is_empty()))
    } else {
        Ok(false)
    }
}

fn is_write_permission_granted(
    network_config: &crate::config::NetworkConfig,
    near_social_account_id: &near_kit::AccountId,
    permission_key_json: serde_json::Value,
    key: String,
) -> color_eyre::eyre::Result<bool> {
    let mut args = serde_json::json!({ "key": key });
    if let (Some(args_map), Some(perm_map)) = (args.as_object_mut(), permission_key_json.as_object())
    {
        for (k, v) in perm_map {
            args_map.insert(k.clone(), v.clone());
        }
    }
    let result = blocking_view_function(
        network_config,
        near_social_account_id,
        "is_write_permission_granted",
        serde_json::to_vec(&args)
            .wrap_err("Internal error: could not serialize `is_write_permission_granted` input args")?,
        near_kit::Finality::Final.into(),
    )
    .wrap_err("Failed to fetch query for view method: 'is_write_permission_granted'")?;
    let serde_call_result: serde_json::Value = result
        .parse_result_from_json::<serde_json::Value>()
        .wrap_err("Failed to parse is_write_permission_granted return value")?;
    Ok(serde_call_result.as_bool().expect("Unexpected response"))
}

#[tracing::instrument(name = "Update the required deposit ...", skip_all)]
fn get_deposit(
    network_config: &crate::config::NetworkConfig,
    signer_account_id: &near_kit::AccountId,
    signer_public_key: &near_kit::PublicKey,
    account_id: &near_kit::AccountId,
    key: &str,
    near_social_account_id: &near_kit::AccountId,
    required_deposit: near_token::NearToken,
) -> color_eyre::eyre::Result<near_token::NearToken> {
    tracing::info!(target: "near_teach_me", "Update the required deposit ...");

    let nk_access_key_view = blocking_view_access_key(
        network_config,
        signer_account_id,
        signer_public_key,
        near_kit::Finality::Final.into(),
    )
    .wrap_err_with(|| {
        format!("Failed to fetch query 'view access key' for <{signer_public_key}>")
    })?;
    let is_signer_access_key_full_access = matches!(
        nk_access_key_view.permission,
        near_kit::AccessKeyPermissionView::FullAccess
    );

    let is_write_permission_granted_to_public_key = is_write_permission_granted(
        network_config,
        near_social_account_id,
        serde_json::json!({ "public_key": signer_public_key.to_string() }),
        format!("{account_id}/{key}"),
    )?;

    let is_write_permission_granted_to_signer = is_write_permission_granted(
        network_config,
        near_social_account_id,
        serde_json::json!({ "predecessor_id": signer_account_id.to_string() }),
        format!("{account_id}/{key}"),
    )?;

    let deposit = if is_signer_access_key_full_access
        || is_signer_access_key_function_call_access_can_call_set_on_social_db_account(
            near_social_account_id,
            &nk_access_key_view.permission,
        )? {
        if is_write_permission_granted_to_public_key || is_write_permission_granted_to_signer {
            if required_deposit.is_zero() {
                near_token::NearToken::from_near(0)
            } else if is_signer_access_key_full_access {
                required_deposit
            } else {
                color_eyre::eyre::bail!("ERROR: Social DB requires more storage deposit, but we cannot cover it when signing transaction with a Function Call only access key")
            }
        } else if signer_account_id == account_id {
            if is_signer_access_key_full_access {
                if required_deposit.is_zero() {
                    near_token::NearToken::from_yoctonear(1)
                } else {
                    required_deposit
                }
            } else if required_deposit.is_zero() {
                required_deposit
            } else {
                color_eyre::eyre::bail!("ERROR: Social DB requires more storage deposit, but we cannot cover it when signing transaction with a Function Call only access key")
            }
        } else {
            color_eyre::eyre::bail!(
                "ERROR: the signer is not allowed to modify the components of this account_id."
            )
        }
    } else {
        color_eyre::eyre::bail!("ERROR: signer access key cannot be used to sign a transaction to update components in Social DB.")
    };
    Ok(deposit)
}

#[tracing::instrument(name = "Getting data about a remote profile ...", skip_all)]
fn get_remote_profile(
    network_config: &crate::config::NetworkConfig,
    near_social_account_id: &near_kit::AccountId,
    account_id: near_kit::AccountId,
) -> color_eyre::eyre::Result<serde_json::Value> {
    tracing::info!(target: "near_teach_me", "Getting data about a remote profile ...");
    let result = blocking_view_function(
        network_config,
        near_social_account_id,
        "get",
        serde_json::to_vec(&serde_json::json!({
            "keys": vec![format!("{account_id}/profile/**")],
        }))?,
        near_kit::Finality::Final.into(),
    )
    .wrap_err_with(|| {
        format!("Failed to fetch query for view method: 'get {account_id}/profile/**' (contract <{}> on network <{}>)",
            near_social_account_id,
            network_config.network_name
        )
    })?;
    match result
        .parse_result_from_json::<crate::types::socialdb::SocialDb>()
        .wrap_err_with(|| {
            format!("Failed to parse view function call return value for {account_id}/profile.")
        })?
        .accounts
        .get(&account_id) {
            Some(account_profile) => Ok(serde_json::to_value(account_profile.profile.clone())?),
            None => Ok(serde_json::Value::Null)
        }
}
