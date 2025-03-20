use color_eyre::eyre::WrapErr;
use serde_json::json;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::AccountPropertiesContext)]
#[interactive_clap(output_context = SignerAccountIdContext)]
pub struct SignerAccountId {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the signer account ID?
    signer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct SignerAccountIdContext {
    global_context: crate::GlobalContext,
    account_properties: super::AccountProperties,
    signer_account_id: near_primitives::types::AccountId,
    on_before_sending_transaction_callback:
        crate::transaction_signature_options::OnBeforeSendingTransactionCallback,
}

impl SignerAccountIdContext {
    pub fn from_previous_context(
        previous_context: super::AccountPropertiesContext,
        scope: &<SignerAccountId as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            account_properties: previous_context.account_properties,
            signer_account_id: scope.signer_account_id.clone().into(),
            on_before_sending_transaction_callback: previous_context
                .on_before_sending_transaction_callback,
        })
    }
}

impl From<SignerAccountIdContext> for crate::commands::ActionContext {
    fn from(item: SignerAccountIdContext) -> Self {
        let global_context = item.global_context.clone();

        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let new_account_id = item.account_properties.new_account_id.clone();
                let signer_id = item.signer_account_id.clone();

                move |network_config| {
                    if new_account_id.as_str().chars().count()
                        < super::MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH
                        && new_account_id.is_top_level()
                    {
                        return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                            "\nAccount <{}> has <{}> character count. Only REGISTRAR_ACCOUNT_ID account can create new top level accounts that are shorter than MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH (32) characters.",
                            new_account_id, new_account_id.as_str().chars().count()
                        ));
                    }
                    if !item.global_context.offline {
                        validate_new_account_id(network_config, &new_account_id)?;
                    }
                    let (actions, receiver_id) = if new_account_id.is_sub_account_of(&signer_id) {
                        (vec![
                                near_primitives::transaction::Action::CreateAccount(
                                    near_primitives::transaction::CreateAccountAction {},
                                ),
                                near_primitives::transaction::Action::Transfer(
                                    near_primitives::transaction::TransferAction {
                                        deposit: item.account_properties.initial_balance.as_yoctonear(),
                                    },
                                ),
                                near_primitives::transaction::Action::AddKey(
                                    Box::new(near_primitives::transaction::AddKeyAction {
                                        public_key: item.account_properties.public_key.clone(),
                                        access_key: near_primitives::account::AccessKey {
                                            nonce: 0,
                                            permission:
                                                near_primitives::account::AccessKeyPermission::FullAccess,
                                        },
                                    }),
                                ),
                            ],
                        new_account_id.clone())
                    } else {
                        let args = serde_json::to_vec(&json!({
                            "new_account_id": new_account_id.clone().to_string(),
                            "new_public_key": item.account_properties.public_key.to_string()
                        }))?;

                        if let Some(linkdrop_account_id) = &network_config.linkdrop_account_id {
                            if new_account_id.is_sub_account_of(linkdrop_account_id)
                                || new_account_id.is_top_level()
                            {
                                (
                                    vec![near_primitives::transaction::Action::FunctionCall(
                                        Box::new(
                                            near_primitives::transaction::FunctionCallAction {
                                                method_name: "create_account".to_string(),
                                                args,
                                                gas: crate::common::NearGas::from_tgas(30).as_gas(),
                                                deposit: item
                                                    .account_properties
                                                    .initial_balance
                                                    .as_yoctonear(),
                                            },
                                        ),
                                    )],
                                    linkdrop_account_id.clone(),
                                )
                            } else {
                                return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                                    "\nSigner account <{}> does not have permission to create account <{}>.",
                                    signer_id,
                                    new_account_id
                                ));
                            }
                        } else {
                            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                                "\nAccount <{}> cannot be created on network <{}> because a <linkdrop_account_id> is not specified in the configuration file.\nYou can learn about working with the configuration file: https://github.com/near/near-cli-rs/blob/master/docs/README.en.md#config. \nExample <linkdrop_account_id> in configuration file: https://github.com/near/near-cli-rs/blob/master/docs/media/linkdrop account_id.png",
                                new_account_id,
                                network_config.network_name
                            ));
                        }
                    };

                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: signer_id.clone(),
                        receiver_id,
                        actions,
                    })
                }
            });

        let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback =
            std::sync::Arc::new({
                let credentials_home_dir = global_context.config.credentials_home_dir.clone();

                move |outcome_view, _network_config| {
                    crate::common::update_used_account_list_as_signer(
                        &credentials_home_dir,
                        &outcome_view.transaction.receiver_id,
                    );
                    Ok(())
                }
            });

        Self {
            global_context,
            interacting_with_account_ids: vec![
                item.signer_account_id,
                item.account_properties.new_account_id,
            ],
            get_prepopulated_transaction_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: item.on_before_sending_transaction_callback,
            on_after_sending_transaction_callback,
        }
    }
}

impl SignerAccountId {
    fn input_signer_account_id(
        context: &super::AccountPropertiesContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        let parent_account_id =
            crate::types::account_id::AccountId::get_parent_account_id_from_sub_account(
                context.account_properties.new_account_id.clone().into(),
            );
        if !parent_account_id.0.is_top_level() {
            Ok(Some(parent_account_id))
        } else {
            crate::common::input_signer_account_id_from_used_account_list(
                &context.global_context.config.credentials_home_dir,
                "What is the signer account ID?",
            )
        }
    }
}

#[tracing::instrument(name = "Validation new account_id ...", skip_all)]
fn validate_new_account_id(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
) -> crate::CliResult {
    let account_state =
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(crate::common::get_account_state(
                network_config,
                account_id,
                near_primitives::types::BlockReference::latest(),
            ));
    match account_state {
        Ok(_) => {
            color_eyre::eyre::Result::Err(sysexits::ExitCode::Protocol) // 76 - The remote system returned something that was “not possible” during a protocol exchange.
                .wrap_err_with(|| format!(
                    "\nAccount <{}> already exists in network <{}>. Therefore, it is not possible to create an account with this name.",
                    account_id,
                    network_config.network_name
                ))
        }
        Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(
            near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(
                near_jsonrpc_primitives::types::query::RpcQueryError::UnknownAccount { .. },
            ),
        )) => Ok(()),
        Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(_)) => {
            tracing::warn!(
                parent: &tracing::Span::none(),
                "Transport error.{}",
                crate::common::indent_payload(
                    "\nIt is currently possible to continue creating an account offline.\nYou can sign and send the created transaction later.\n"
                )
            );
            Ok(())
        }
        Err(err) => color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(err.to_string())),
    }
}
