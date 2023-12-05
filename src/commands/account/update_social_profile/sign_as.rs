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
        let data = item.data.clone();

        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            Arc::new({
                move |network_config| {
                    let contract_account_id =
                        network_config.get_near_social_account_id_from_network()?;
                    let mut prepopulated_transaction = crate::commands::PrepopulatedTransaction {
                        signer_id: signer_id.clone(),
                        receiver_id: contract_account_id.clone(),
                        actions: vec![],
                    };

                    let json_rpc_client = network_config.json_rpc_client();

                    let local_profile: serde_json::Value = serde_json::from_slice(&data)?;
                    let remote_profile = match json_rpc_client
                        .blocking_call_view_function(
                            &contract_account_id,
                            "get",
                            serde_json::to_vec(&serde_json::json!({
                                "keys": vec![format!("{account_id}/profile/**")],
                            }))?,
                            near_primitives::types::Finality::Final.into(),
                        )
                        .wrap_err_with(|| {
                            format!("Failed to fetch query for view method: 'get {account_id}/profile/**' (contract <{}> on network <{}>)",
                                contract_account_id,
                                network_config.network_name
                            )
                        })?
                        .parse_result_from_json::<near_socialdb_client::types::socialdb_types::SocialDb>()
                        .wrap_err_with(|| {
                            format!("Failed to parse view function call return value for {account_id}/profile.")
                        })?
                        .accounts
                        .get(&account_id) {
                            Some(account_profile) => serde_json::to_value(account_profile.profile.clone())?,
                            None => serde_json::Value::Null
                        };

                    let deposit = tokio::runtime::Runtime::new().unwrap().block_on(
                        near_socialdb_client::required_deposit(
                            &json_rpc_client,
                            &contract_account_id,
                            &account_id,
                            &local_profile,
                            Some(&remote_profile),
                        ),
                    )?;

                    let new_social_db_state =
                        near_socialdb_client::types::socialdb_types::SocialDb {
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

                    prepopulated_transaction.actions =
                        vec![near_primitives::transaction::Action::FunctionCall(
                            near_primitives::transaction::FunctionCallAction {
                                method_name: "set".to_string(),
                                args,
                                gas: crate::common::NearGas::from_tgas(300).as_gas(),
                                deposit: deposit.as_yoctonear(),
                            },
                        )];

                    Ok(prepopulated_transaction)
                }
            });

        let on_before_signing_callback: crate::commands::OnBeforeSigningCallback = Arc::new({
            let signer_account_id = item.signer_account_id.clone();
            let account_id = item.account_id.clone();
            move |prepopulated_unsigned_transaction, network_config| {
                let json_rpc_client = network_config.json_rpc_client();

                if let near_primitives::transaction::Action::FunctionCall(action) =
                    &mut prepopulated_unsigned_transaction.actions[0]
                {
                    action.deposit = tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(near_socialdb_client::get_deposit(
                            &json_rpc_client,
                            &signer_account_id,
                            &prepopulated_unsigned_transaction.public_key,
                            &account_id,
                            "profile",
                            &prepopulated_unsigned_transaction.receiver_id,
                            near_token::NearToken::from_yoctonear(action.deposit),
                        ))?
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
            on_after_getting_network_callback,
            on_before_signing_callback,
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
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
                let networks: Vec<String> = context
                    .global_context
                    .config
                    .network_connection
                    .iter()
                    .map(|(_, network_config)| network_config.network_name.clone())
                    .collect();
                eprintln!(
                    "\nThe account <{signer_account_id}> does not exist on [{}] networks.",
                    networks.join(", ")
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
