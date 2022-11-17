use dialoguer::{console::Term, theme::ColorfulTheme, Input, Select};
use serde_json::json;
use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::commands::account::create_account::CreateAccountContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignerAccountId {
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    ///What is the signer account ID?
    signer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

impl SignerAccountId {
    pub fn from_cli(
        optional_clap_variant: Option<<SignerAccountId as interactive_clap::ToCli>::CliVariant>,
        context: crate::commands::account::create_account::CreateAccountContext,
    ) -> color_eyre::eyre::Result<Option<Self>> {
        let signer_account_id: crate::types::account_id::AccountId = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.signer_account_id)
        {
            Some(cli_signer_account_id) => cli_signer_account_id,
            None => {
                let owner_account_id = context
                    .new_account_id
                    .clone()
                    .get_owner_account_id_from_sub_account();
                if !owner_account_id.0.is_top_level() {
                    owner_account_id
                } else {
                    Self::input_signer_account_id(&context)?
                }
            }
        };
        let network_config = crate::network_for_transaction::NetworkForTransactionArgs::from_cli(
            optional_clap_variant.and_then(|clap_variant| {
                clap_variant.network_config.map(
                    |ClapNamedArgNetworkForTransactionArgsForSignerAccountId::NetworkConfig(
                        cli_network_config,
                    )| cli_network_config,
                )
            }),
            (context.config,),
        )?;
        let network_config = if let Some(value) = network_config {
            value
        } else {
            return Ok(None);
        };
        Ok(Some(Self {
            signer_account_id,
            network_config,
        }))
    }

    fn input_signer_account_id(
        context: &crate::commands::account::create_account::CreateAccountContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        loop {
            let signer_account_id: crate::types::account_id::AccountId = Input::new()
                .with_prompt("What is the signer account ID?")
                .interact_text()?;
            let optional_account_view = 'block: {
                for network in context.config.networks.iter() {
                    match tokio::runtime::Runtime::new().unwrap().block_on(
                        crate::common::get_account_state(
                            network.1.clone(),
                            signer_account_id.clone().into(),
                            near_primitives::types::Finality::Final.into(),
                        ),
                    ) {
                        Ok(optional_account_view) => {
                            if optional_account_view.is_some() {
                                break 'block optional_account_view;
                            }
                        }
                        Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(_)) => {
                            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!("Failed to lookup address information: nodename nor servname provided, or no network connection. So now there is no way to check if <{}> exists.", signer_account_id));
                        },
                        Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(near_jsonrpc_primitives::types::query::RpcQueryError::UnknownAccount{requested_account_id:_, block_hash:_, block_height:_}))) => {},
                        Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(_)) => println!("Unable to verify the existence of account <{}> on network <{}>", signer_account_id, network.1.network_name),
                    }
                }
                None
            };
            if optional_account_view.is_none() {
                println!("\nThe account <{}> does not yet exist.", &signer_account_id);
                let choose_input = vec![
                    "Yes, I want to enter a new name for signer_account_id.",
                    "No, I want to use this name for signer_account_id.",
                ];
                let select_choose_input = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Do you want to enter a new name for signer_account_id?")
                    .items(&choose_input)
                    .default(0)
                    .interact_on_opt(&Term::stderr())?;
                if matches!(select_choose_input, Some(1)) {
                    break Ok(signer_account_id);
                }
            } else {
                break Ok(signer_account_id);
            }
        }
    }

    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: super::AccountProperties,
        storage_properties: Option<super::StorageProperties>,
    ) -> crate::CliResult {
        let network_config = self.network_config.get_network_config(config.clone());

        match crate::common::get_account_state(
            network_config.clone(),
            self.signer_account_id.clone().into(),
            near_primitives::types::Finality::Final.into(),
        )
        .await
        {
            Ok(optional_account_view) => {
                if optional_account_view.is_none() {
                    return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                        "\nSigner account <{}> does not yet exist on the network <{}>.",
                        self.signer_account_id,
                        network_config.network_name
                    ));
                }
            }
            Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(_)) => {
                return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!("Failed to lookup address information: nodename nor servname provided, or no network connection. So now there is no way to check if <{}> exists.", self.signer_account_id));
            }
            Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(
                near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(
                    near_jsonrpc_primitives::types::query::RpcQueryError::UnknownAccount {
                        requested_account_id,
                        ..
                    },
                ),
            )) => {
                return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                    "\nSigner account <{}> does not yet exist on the network <{}>.",
                    requested_account_id,
                    network_config.network_name
                ));
            }
            Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(_)) => println!(
                "Unable to verify the existence of account <{}> on network <{}>",
                self.signer_account_id, network_config.network_name
            ),
        }

        if account_properties.new_account_id.as_str().chars().count()
            < super::MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH
            && !account_properties.new_account_id.as_str().contains('.')
        {
            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "\nAccount <{}> has <{}> character count. Only REGISTRAR_ACCOUNT_ID account can create new top level accounts that are shorter than MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH (32) characters.",
                account_properties.new_account_id, account_properties.new_account_id.as_str().chars().count()
            ));
        }
        match crate::common::get_account_state(
            network_config.clone(),
            account_properties.new_account_id.clone(),
            near_primitives::types::Finality::Final.into(),
        )
        .await
        {
            Ok(optional_account_view) => {
                if optional_account_view.is_some() {
                    return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                        "\nAccount <{}> already exists in network <{}>.",
                        account_properties.new_account_id,
                        network_config.network_name
                    ));
                }
            }
            Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(_)) => {
                return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!("Failed to lookup address information: nodename nor servname provided, or no network connection. So now there is no way to check if <{}> exists.", self.signer_account_id));
            }
            Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(
                near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(
                    near_jsonrpc_primitives::types::query::RpcQueryError::UnknownAccount {
                        requested_account_id: _,
                        block_hash: _,
                        block_height: _,
                    },
                ),
            )) => {}
            Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(_)) => println!(
                "Unable to verify the existence of account <{}> on network <{}>",
                self.signer_account_id, network_config.network_name
            ),
        }

        let (actions, receiver_id) = if account_properties
            .new_account_id
            .clone()
            .is_sub_account_of(&self.signer_account_id.0)
        {
            (
                vec![
                    near_primitives::transaction::Action::CreateAccount(
                        near_primitives::transaction::CreateAccountAction {},
                    ),
                    near_primitives::transaction::Action::Transfer(
                        near_primitives::transaction::TransferAction {
                            deposit: account_properties.initial_balance.to_yoctonear(),
                        },
                    ),
                    near_primitives::transaction::Action::AddKey(
                        near_primitives::transaction::AddKeyAction {
                            public_key: near_crypto::PublicKey::from_str(
                                &account_properties.public_key.to_string(),
                            )?,
                            access_key: near_primitives::account::AccessKey {
                                nonce: 0,
                                permission:
                                    near_primitives::account::AccessKeyPermission::FullAccess,
                            },
                        },
                    ),
                ],
                account_properties.new_account_id.clone(),
            )
        } else if !account_properties
            .new_account_id
            .clone()
            .is_sub_account_of(&self.signer_account_id.0)
            && account_properties
                .new_account_id
                .as_str()
                .split('.')
                .count()
                > 2
        {
            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "\nSigner account <{}> does not have permission to create account <{}>.",
                self.signer_account_id,
                account_properties.new_account_id
            ));
        } else {
            let args = json!({
                "new_account_id": account_properties.new_account_id.clone().to_string(),
                "new_public_key": account_properties.public_key.to_string()
            })
            .to_string()
            .into_bytes();

            match network_config.clone().linkdrop_account_id {
            Some(linkdrop_account_id) => (
                vec![near_primitives::transaction::Action::FunctionCall(
                    near_primitives::transaction::FunctionCallAction {
                        method_name: "create_account".to_string(),
                        args,
                        gas: crate::common::NearGas::from_str("30 TeraGas")
                            .unwrap()
                            .inner,
                        deposit: account_properties.initial_balance.to_yoctonear(),
                    },
                )],
                linkdrop_account_id,
            ),
            None => return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                    "\nAccount <{}> cannot be created on network <{}> because a <linkdrop_account_id> is not specified in the configuration file.\nYou can learn about working with the configuration file: https://github.com/near/near-cli-rs/blob/master/docs/README.en.md#config. \nExample <linkdrop_account_id> in configuration file: https://github.com/near/near-cli-rs/blob/master/docs/media/linkdrop account_id.png",
                    account_properties.new_account_id,
                    network_config.network_name
                ))
            }
        };

        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self.signer_account_id.0.clone(),
            public_key: account_properties.clone().public_key,
            nonce: 0,
            receiver_id,
            block_hash: Default::default(),
            actions,
        };

        let storage_message = match storage_properties.clone() {
            Some(properties) => match properties.storage {
                #[cfg(target_os = "macos")]
                super::add_key::autogenerate_new_keypair::SaveModeDiscriminants::SaveToMacosKeychain => {
                    super::add_key::autogenerate_new_keypair::SaveMode::save_access_key_to_macos_keychain(
                        network_config,
                        account_properties.clone(),
                        storage_properties.clone(),
                    )
                    .await?
                }
                super::add_key::autogenerate_new_keypair::SaveModeDiscriminants::SaveToKeychain => {
                    super::add_key::autogenerate_new_keypair::SaveMode::save_access_key_to_keychain(
                        config.clone(),
                        network_config,
                        account_properties.clone(),
                        storage_properties.clone(),
                    )
                    .await?
                }
                super::add_key::autogenerate_new_keypair::SaveModeDiscriminants::PrintToTerminal => {
                    super::add_key::autogenerate_new_keypair::SaveMode::print_access_key_to_terminal(
                        storage_properties.clone(),
                    )?
                }
            },
            None => String::new(),
        };

        match crate::transaction_signature_options::sign_with(
            self.network_config.clone(),
            prepopulated_unsigned_transaction,
            config.clone(),
        )
        .await?
        {
            Some(transaction_info) => match transaction_info.status {
                near_primitives::views::FinalExecutionStatus::SuccessValue(ref value) => {
                    if value == b"false" {
                        println!(
                            "The new account <{}> could not be created successfully.",
                            account_properties.new_account_id
                        );
                    } else {
                        println!(
                            "New account <{}> created successfully.",
                            account_properties.new_account_id
                        );
                    }
                    println!("Transaction ID: {id}\nTo see the transaction in the transaction explorer, please open this url in your browser:\n{path}{id}\n",
                                id=transaction_info.transaction_outcome.id,
                                path=self.network_config.get_network_config(config).explorer_transaction_url
                            );
                    if storage_properties.is_some() {
                        println!("{}\n", storage_message);
                    }
                    Ok(())
                }
                _ => {
                    crate::common::print_transaction_status(
                        transaction_info,
                        self.network_config.get_network_config(config),
                    )?;
                    if storage_properties.is_some() {
                        println!("{}\n", storage_message);
                    }
                    Ok(())
                }
            },
            None => Ok(()),
        }
    }
}
