use std::str::FromStr;

use serde_json::json;

use inquire::{CustomType, Select};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::account::create_account::CreateAccountContext)]
#[interactive_clap(output_context = crate::commands::ActionContext)]
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
    config: crate::config::Config,
    account_properties: super::super::AccountProperties,
    signer_account_id: near_primitives::types::AccountId,
}

impl SignerAccountIdContext {
    pub fn from_previous_context(
        previous_context: crate::commands::account::create_account::CreateAccountContext,
        scope: &<SignerAccountId as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            account_properties: previous_context.account_properties,
            signer_account_id: scope.signer_account_id.clone().into(),
        })
    }
}

impl From<SignerAccountIdContext> for crate::commands::ActionContext {
    fn from(item: SignerAccountIdContext) -> Self {
        let receiver_account_id: near_primitives::types::AccountId =
            item.account_properties.new_account_id.clone().into();
        let signer_account_id: near_primitives::types::AccountId = item.signer_account_id.clone();
        let config = item.config.clone();

        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let new_account_id: near_primitives::types::AccountId =
                    item.account_properties.new_account_id.clone().into();
                let signer_account_id = item.signer_account_id.clone();

                move |prepopulated_unsigned_transaction, network_config| {
                    tokio::runtime::Runtime::new().unwrap().block_on(
                        validate_signer_account_id(&network_config, &signer_account_id.clone()),
                    )?;

                    if new_account_id.as_str().chars().count()
                        < super::MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH
                        && new_account_id.is_top_level()
                    {
                        return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                            "\nAccount <{}> has <{}> character count. Only REGISTRAR_ACCOUNT_ID account can create new top level accounts that are shorter than MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH (32) characters.",
                            new_account_id, new_account_id.as_str().chars().count()
                        ));
                    }
                    tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(validate_new_account_id(&network_config, &new_account_id))?;

                    let (actions, receiver_id) = if new_account_id
                        .is_sub_account_of(&signer_account_id)
                    {
                        (
                            vec![
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
                            new_account_id.clone(),
                        )
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
                                    signer_account_id,
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

                    prepopulated_unsigned_transaction.receiver_id = receiver_id;
                    prepopulated_unsigned_transaction.actions = actions;

                    // let storage_message = match item.storage_properties.clone() {
                    //     Some(properties) => {
                    //         let key_pair_properties_buf =
                    //             serde_json::to_string(&properties.key_pair_properties)?;
                    //         match properties.storage {
                    //             #[cfg(target_os = "macos")]
                    //             super::add_key::autogenerate_new_keypair::SaveModeDiscriminants::SaveToMacosKeychain => {
                    //                 // super::add_key::autogenerate_new_keypair::SaveMode::save_access_key_to_macos_keychain(
                    //                 //     network_config.clone(),
                    //                 //     item.account_properties.clone(),
                    //                 //     item.storage_properties.clone(),
                    //                 // )?
                    //                 crate::common::save_access_key_to_macos_keychain(
                    //                     network_config.clone(),
                    //                     &key_pair_properties_buf,
                    //                     &properties.key_pair_properties.public_key_str,
                    //                     &new_account_id,
                    //                 )
                    //                 .map_err(|err| {
                    //                     color_eyre::Report::msg(format!(
                    //                         "Failed to save a file with access key: {}",
                    //                         err
                    //                     ))
                    //                 })?
                    //                                 }
                    //             super::add_key::autogenerate_new_keypair::SaveModeDiscriminants::SaveToKeychain => {
                    //                 // super::add_key::autogenerate_new_keypair::SaveMode::save_access_key_to_keychain(
                    //                 //     item.config.clone(),
                    //                 //     network_config.clone(),
                    //                 //     item.account_properties.clone(),
                    //                 //     item.storage_properties.clone(),
                    //                 // )?
                    //                 crate::common::save_access_key_to_keychain(
                    //                     network_config.clone(),
                    //                     credentials_home_dir.clone(),
                    //                     &key_pair_properties_buf,
                    //                     &properties.key_pair_properties.public_key_str,
                    //                     &new_account_id,
                    //                 )
                    //                 .map_err(|err| {
                    //                     color_eyre::Report::msg(format!(
                    //                         "Failed to save a file with access key: {}",
                    //                         err
                    //                     ))
                    //                 })?
                    //             }
                    //             super::add_key::autogenerate_new_keypair::SaveModeDiscriminants::PrintToTerminal => {
                    //                 super::add_key::autogenerate_new_keypair::SaveMode::print_access_key_to_terminal(
                    //                     item.storage_properties.clone(),
                    //                 )?
                    //             }
                    //         }
                    //     }
                    //     None => String::new(),
                    // };

                    // match crate::transaction_signature_options::sign_with(
                    //     network_config.clone(),
                    //     prepopulated_unsigned_transaction,
                    //     item.config.clone(),
                    // )
                    // .await?
                    // {
                    //     Some(transaction_info) => match transaction_info.status {
                    //         near_primitives::views::FinalExecutionStatus::SuccessValue(
                    //             ref value,
                    //         ) => {
                    //             if value == b"false" {
                    //                 println!(
                    //                     "The new account <{}> could not be created successfully.",
                    //                     account_properties.new_account_id
                    //                 );
                    //             } else {
                    //                 println!(
                    //                     "New account <{}> created successfully.",
                    //                     account_properties.new_account_id
                    //                 );
                    //             }
                    //             println!("Transaction ID: {id}\nTo see the transaction in the transaction explorer, please open this url in your browser:\n{path}{id}\n",
                    //                         id=transaction_info.transaction_outcome.id,
                    //                         path=self.network_config.get_network_config(config).explorer_transaction_url
                    //                     );
                    //             if storage_properties.is_some() {
                    //                 println!("{}\n", storage_message);
                    //             }
                    //             Ok(())
                    //         }
                    //         _ => {
                    //             crate::common::print_transaction_status(
                    //                 transaction_info,
                    //                 self.network_config.get_network_config(config),
                    //             )?;
                    //             if storage_properties.is_some() {
                    //                 println!("{}\n", storage_message);
                    //             }
                    //             Ok(())
                    //         }
                    //     },
                    //     None => Ok(()),
                    // }

                    Ok(())
                }
            });

        Self {
            config,
            signer_account_id,
            receiver_account_id,
            actions: vec![],
            on_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }
    }
}

// impl interactive_clap::FromCli for SignerAccountId {
//     type FromCliContext = crate::commands::account::create_account::CreateAccountContext;
//     type FromCliError = color_eyre::eyre::Error;

//     fn from_cli(
//         optional_clap_variant: Option<<SignerAccountId as interactive_clap::ToCli>::CliVariant>,
//         context: Self::FromCliContext,
//     ) -> Result<Option<Self>, Self::FromCliError>
//     where
//         Self: Sized + interactive_clap::ToCli,
//     {
//         let signer_account_id: crate::types::account_id::AccountId = match optional_clap_variant
//             .clone()
//             .and_then(|clap_variant| clap_variant.signer_account_id)
//         {
//             Some(cli_signer_account_id) => cli_signer_account_id,
//             None => Self::input_signer_account_id(&context)?,
//         };

//         let new_context_scope = InteractiveClapContextScopeForSignerAccountId {
//             signer_account_id: signer_account_id.clone(),
//         };
//         let new_context =
//             SignerAccountIdContext::from_previous_context(context, &new_context_scope)?;

//         let network_config = crate::network_for_transaction::NetworkForTransactionArgs::from_cli(
//             optional_clap_variant.and_then(|clap_variant| {
//                 clap_variant.network_config.map(
//                     |ClapNamedArgNetworkForTransactionArgsForSignerAccountId::NetworkConfig(
//                         cli_network_config,
//                     )| cli_network_config,
//                 )
//             }),
//             new_context.into(),
//         )?;
//         let network_config = if let Some(value) = network_config {
//             value
//         } else {
//             return Ok(None);
//         };
//         Ok(Some(Self {
//             signer_account_id,
//             network_config,
//         }))
//     }
// }

impl SignerAccountId {
    fn input_signer_account_id(
        context: &crate::commands::account::create_account::CreateAccountContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        let parent_account_id = context
            .account_properties
            .new_account_id
            .clone()
            .get_parent_account_id_from_sub_account();
        if !parent_account_id.0.is_top_level() {
            if is_account_exist(&context, parent_account_id.clone().into()) {
                Ok(parent_account_id)
            } else {
                Self::input_account_id(&context)
            }
        } else {
            Self::input_account_id(&context)
        }
    }

    fn input_account_id(
        context: &crate::commands::account::create_account::CreateAccountContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        loop {
            let signer_account_id: crate::types::account_id::AccountId =
                CustomType::new("What is the signer account ID?").prompt()?;
            if !is_account_exist(context, signer_account_id.clone().into()) {
                println!("\nThe account <{}> does not yet exist.", &signer_account_id);
                #[derive(strum_macros::Display)]
                enum ConfirmOptions {
                    #[strum(to_string = "Yes, I want to enter a new name for signer_account_id.")]
                    Yes,
                    #[strum(to_string = "No, I want to use this name for signer_account_id.")]
                    No,
                }
                let select_choose_input = Select::new(
                    "Do you want to enter a new name for signer_account_id?",
                    vec![ConfirmOptions::Yes, ConfirmOptions::No],
                )
                .prompt()?;
                if let ConfirmOptions::No = select_choose_input {
                    return Ok(signer_account_id);
                }
            } else {
                return Ok(signer_account_id);
            }
        }
    }

    // pub async fn process(
    //     &self,
    //     config: crate::config::Config,
    //     account_properties: super::super::AccountProperties,
    //     storage_properties: Option<super::StorageProperties>,
    // ) -> crate::CliResult {
    //     let network_config = self.network_config.get_network_config(config.clone());

    //     validate_signer_account_id(&network_config, &self.signer_account_id.clone().into()).await?;

    //     if account_properties.new_account_id.as_str().chars().count()
    //         < super::MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH
    //         && account_properties.new_account_id.is_top_level()
    //     {
    //         return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
    //             "\nAccount <{}> has <{}> character count. Only REGISTRAR_ACCOUNT_ID account can create new top level accounts that are shorter than MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH (32) characters.",
    //             account_properties.new_account_id, account_properties.new_account_id.as_str().chars().count()
    //         ));
    //     }
    //     validate_new_account_id(&network_config, &account_properties.new_account_id).await?;

    //     let (actions, receiver_id) = if account_properties
    //         .new_account_id
    //         .is_sub_account_of(&self.signer_account_id.0)
    //     {
    //         (
    //             vec![
    //                 near_primitives::transaction::Action::CreateAccount(
    //                     near_primitives::transaction::CreateAccountAction {},
    //                 ),
    //                 near_primitives::transaction::Action::Transfer(
    //                     near_primitives::transaction::TransferAction {
    //                         deposit: account_properties.initial_balance.to_yoctonear(),
    //                     },
    //                 ),
    //                 near_primitives::transaction::Action::AddKey(
    //                     near_primitives::transaction::AddKeyAction {
    //                         public_key: account_properties.public_key.clone(),
    //                         access_key: near_primitives::account::AccessKey {
    //                             nonce: 0,
    //                             permission:
    //                                 near_primitives::account::AccessKeyPermission::FullAccess,
    //                         },
    //                     },
    //                 ),
    //             ],
    //             account_properties.new_account_id.clone(),
    //         )
    //     } else {
    //         let args = json!({
    //             "new_account_id": account_properties.new_account_id.clone().to_string(),
    //             "new_public_key": account_properties.public_key.to_string()
    //         })
    //         .to_string()
    //         .into_bytes();

    //         if let Some(linkdrop_account_id) = &network_config.linkdrop_account_id {
    //             if account_properties
    //                 .new_account_id
    //                 .is_sub_account_of(linkdrop_account_id)
    //                 || account_properties.new_account_id.is_top_level()
    //             {
    //                 (
    //                     vec![near_primitives::transaction::Action::FunctionCall(
    //                         near_primitives::transaction::FunctionCallAction {
    //                             method_name: "create_account".to_string(),
    //                             args,
    //                             gas: crate::common::NearGas::from_str("30 TeraGas")
    //                                 .unwrap()
    //                                 .inner,
    //                             deposit: account_properties.initial_balance.to_yoctonear(),
    //                         },
    //                     )],
    //                     linkdrop_account_id.clone(),
    //                 )
    //             } else {
    //                 return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
    //                     "\nSigner account <{}> does not have permission to create account <{}>.",
    //                     self.signer_account_id,
    //                     account_properties.new_account_id
    //                 ));
    //             }
    //         } else {
    //             return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
    //                 "\nAccount <{}> cannot be created on network <{}> because a <linkdrop_account_id> is not specified in the configuration file.\nYou can learn about working with the configuration file: https://github.com/near/near-cli-rs/blob/master/docs/README.en.md#config. \nExample <linkdrop_account_id> in configuration file: https://github.com/near/near-cli-rs/blob/master/docs/media/linkdrop account_id.png",
    //                 account_properties.new_account_id,
    //                 network_config.network_name
    //             ));
    //         }
    //     };

    //     let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
    //         signer_id: self.signer_account_id.0.clone(),
    //         public_key: account_properties.clone().public_key,
    //         nonce: 0,
    //         receiver_id,
    //         block_hash: Default::default(),
    //         actions,
    //     };

    //     let storage_message = match storage_properties.clone() {
    //         Some(properties) => match properties.storage {
    //             #[cfg(target_os = "macos")]
    //             super::add_key::autogenerate_new_keypair::SaveModeDiscriminants::SaveToMacosKeychain => {
    //                 super::add_key::autogenerate_new_keypair::SaveMode::save_access_key_to_macos_keychain(
    //                     network_config,
    //                     account_properties.clone(),
    //                     storage_properties.clone(),
    //                 )?
    //             }
    //             super::add_key::autogenerate_new_keypair::SaveModeDiscriminants::SaveToKeychain => {
    //                 super::add_key::autogenerate_new_keypair::SaveMode::save_access_key_to_keychain(
    //                     config.clone(),
    //                     network_config,
    //                     account_properties.clone(),
    //                     storage_properties.clone(),
    //                 )?
    //             }
    //             super::add_key::autogenerate_new_keypair::SaveModeDiscriminants::PrintToTerminal => {
    //                 super::add_key::autogenerate_new_keypair::SaveMode::print_access_key_to_terminal(
    //                     storage_properties.clone(),
    //                 )?
    //             }
    //         },
    //         None => String::new(),
    //     };

    //     match crate::transaction_signature_options::sign_with(
    //         self.network_config.clone(),
    //         prepopulated_unsigned_transaction,
    //         config.clone(),
    //     )
    //     .await?
    //     {
    //         Some(transaction_info) => match transaction_info.status {
    //             near_primitives::views::FinalExecutionStatus::SuccessValue(ref value) => {
    //                 if value == b"false" {
    //                     println!(
    //                         "The new account <{}> could not be created successfully.",
    //                         account_properties.new_account_id
    //                     );
    //                 } else {
    //                     println!(
    //                         "New account <{}> created successfully.",
    //                         account_properties.new_account_id
    //                     );
    //                 }
    //                 println!("Transaction ID: {id}\nTo see the transaction in the transaction explorer, please open this url in your browser:\n{path}{id}\n",
    //                             id=transaction_info.transaction_outcome.id,
    //                             path=self.network_config.get_network_config(config).explorer_transaction_url
    //                         );
    //                 if storage_properties.is_some() {
    //                     println!("{}\n", storage_message);
    //                 }
    //                 Ok(())
    //             }
    //             _ => {
    //                 crate::common::print_transaction_status(
    //                     transaction_info,
    //                     self.network_config.get_network_config(config),
    //                 )?;
    //                 if storage_properties.is_some() {
    //                     println!("{}\n", storage_message);
    //                 }
    //                 Ok(())
    //             }
    //         },
    //         None => Ok(()),
    //     }
    // }
}

fn is_account_exist(
    context: &crate::commands::account::create_account::CreateAccountContext,
    account_id: near_primitives::types::AccountId,
) -> bool {
    for network in context.config.networks.iter() {
        if tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(crate::common::get_account_state(
                network.1.clone(),
                account_id.clone(),
                near_primitives::types::Finality::Final.into(),
            ))
            .is_ok()
        {
            return true;
        }
    }
    false
}

async fn validate_signer_account_id(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
) -> crate::CliResult {
    match crate::common::get_account_state(
        network_config.clone(),
        account_id.clone(),
        near_primitives::types::Finality::Final.into(),
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(
            near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(
                near_jsonrpc_primitives::types::query::RpcQueryError::UnknownAccount {
                    requested_account_id,
                    ..
                },
            ),
        )) => color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
            "Signer account <{}> does not currently exist on network <{}>.",
            requested_account_id,
            network_config.network_name
        )),
        Err(err) => color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(err.to_string())),
    }
}

async fn validate_new_account_id(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
) -> crate::CliResult {
    match crate::common::get_account_state(
            network_config.clone(),
            account_id.clone(),
            near_primitives::types::Finality::Final.into(),
        )
        .await
        {
            Ok(_) => {
                color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                    "\nAccount <{}> already exists in network <{}>. Therefore, it is not possible to create an account with this name.",
                    account_id,
                    network_config.network_name
                ))
            }
            Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(
                near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(
                    near_jsonrpc_primitives::types::query::RpcQueryError::UnknownAccount { .. },
                ),
            )) => {
                Ok(())
            }
            Err(err) => color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(err.to_string())),
        }
}
