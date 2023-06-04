use std::str::FromStr;

use inquire::CustomType;
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

        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let new_account_id: near_primitives::types::AccountId =
                    item.account_properties.new_account_id.clone().into();
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
                                        deposit: item.account_properties.initial_balance.to_yoctonear(),
                                    },
                                ),
                                near_primitives::transaction::Action::AddKey(
                                    near_primitives::transaction::AddKeyAction {
                                        public_key: item.account_properties.public_key.clone(),
                                        access_key: near_primitives::account::AccessKey {
                                            nonce: 0,
                                            permission:
                                                near_primitives::account::AccessKeyPermission::FullAccess,
                                        },
                                    },
                                ),
                            ],
                        new_account_id.clone())
                    } else {
                        let args = json!({
                            "new_account_id": new_account_id.clone().to_string(),
                            "new_public_key": item.account_properties.public_key.to_string()
                        })
                        .to_string()
                        .into_bytes();

                        if let Some(linkdrop_account_id) = &network_config.linkdrop_account_id {
                            if new_account_id.is_sub_account_of(linkdrop_account_id)
                                || new_account_id.is_top_level()
                            {
                                (
                                    vec![near_primitives::transaction::Action::FunctionCall(
                                        near_primitives::transaction::FunctionCallAction {
                                            method_name: "create_account".to_string(),
                                            args,
                                            gas: crate::common::NearGas::from_str("30 TeraGas")
                                                .unwrap()
                                                .inner,
                                            deposit: item
                                                .account_properties
                                                .initial_balance
                                                .to_yoctonear(),
                                        },
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

        Self {
            global_context,
            on_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: item.on_before_sending_transaction_callback,
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }
    }
}

impl SignerAccountId {
    fn input_signer_account_id(
        context: &super::AccountPropertiesContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        let parent_account_id = context
            .account_properties
            .new_account_id
            .clone()
            .get_parent_account_id_from_sub_account();
        if !parent_account_id.0.is_top_level() {
            Ok(Some(parent_account_id))
        } else {
            Self::input_account_id(context)
        }
    }

    fn input_account_id(
        _context: &super::AccountPropertiesContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        let signer_account_id: crate::types::account_id::AccountId =
            CustomType::new("What is the signer account ID?").prompt()?;
        Ok(Some(signer_account_id))
    }
}

fn validate_new_account_id(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
) -> crate::CliResult {
    for _ in 0..3 {
        let account_state = crate::common::get_account_state(
            network_config.clone(),
            account_id.clone(),
            near_primitives::types::BlockReference::latest(),
        );
        if let Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(
            near_jsonrpc_client::errors::RpcTransportError::SendError(_),
        )) = account_state
        {
            eprintln!("Transport error.\nPlease wait. The next try to send this query is happening right now ...");
            std::thread::sleep(std::time::Duration::from_millis(100))
        } else {
            match account_state {
                Ok(_) => {
                    return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "\nAccount <{}> already exists in network <{}>. Therefore, it is not possible to create an account with this name.",
                account_id,
                network_config.network_name
            ));
                }
                Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(
                    near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(
                        near_jsonrpc_primitives::types::query::RpcQueryError::UnknownAccount {
                            ..
                        },
                    ),
                )) => {
                    return Ok(());
                }
                Err(err) => {
                    return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(err.to_string()))
                }
            }
        }
    }
    eprintln!("\nTransport error.\nIt is currently possible to continue creating an account offline.\nYou can sign and send the created transaction later.");
    Ok(())
}
