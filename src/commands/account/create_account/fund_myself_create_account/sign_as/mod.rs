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
                    if optional_owner_account_view(&context, owner_account_id.clone().into())
                        .is_some()
                    {
                        owner_account_id
                    } else {
                        Self::input_signer_account_id(&context)?
                    }
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
            if optional_signer_account_view(context, signer_account_id.clone().into())?.is_none() {
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
                    return Ok(signer_account_id);
                }
            } else {
                return Ok(signer_account_id);
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

        validate_signer_account_id(&network_config, &self.signer_account_id.clone().into()).await?;

        if account_properties.new_account_id.as_str().chars().count()
            < super::MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH
            && !account_properties.new_account_id.as_str().contains('.')
        {
            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "\nAccount <{}> has <{}> character count. Only REGISTRAR_ACCOUNT_ID account can create new top level accounts that are shorter than MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH (32) characters.",
                account_properties.new_account_id, account_properties.new_account_id.as_str().chars().count()
            ));
        }
        validate_new_account_id(&network_config, &account_properties.new_account_id).await?;

        let (actions, receiver_id) = if account_properties
            .new_account_id
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
                            public_key: account_properties.public_key.clone(),
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
        } else {
            let args = json!({
                "new_account_id": account_properties.new_account_id.clone().to_string(),
                "new_public_key": account_properties.public_key.to_string()
            })
            .to_string()
            .into_bytes();

            if let Some(linkdrop_account_id) = &network_config.linkdrop_account_id {
                if account_properties
                    .new_account_id
                    .is_sub_account_of(linkdrop_account_id)
                    || !account_properties.new_account_id.as_str().contains('.')
                {
                    (
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
                        linkdrop_account_id.clone(),
                    )
                } else {
                    return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                        "\nSigner account <{}> does not have permission to create account <{}>.",
                        self.signer_account_id,
                        account_properties.new_account_id
                    ));
                }
            } else {
                return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                    "\nAccount <{}> cannot be created on network <{}> because a <linkdrop_account_id> is not specified in the configuration file.\nYou can learn about working with the configuration file: https://github.com/near/near-cli-rs/blob/master/docs/README.en.md#config. \nExample <linkdrop_account_id> in configuration file: https://github.com/near/near-cli-rs/blob/master/docs/media/linkdrop account_id.png",
                    account_properties.new_account_id,
                    network_config.network_name
                ));
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

fn optional_owner_account_view(
    context: &crate::commands::account::create_account::CreateAccountContext,
    account_id: near_primitives::types::AccountId,
) -> Option<near_primitives::views::AccountView> {
    for network in context.config.networks.iter() {
        if let Ok(optional_account_view) =
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(crate::common::get_account_state(
                    network.1.clone(),
                    account_id.clone(),
                    near_primitives::types::Finality::Final.into(),
                ))
        {
            if optional_account_view.is_some() {
                return optional_account_view;
            }
        }
    }
    None
}

fn optional_signer_account_view(
    context: &crate::commands::account::create_account::CreateAccountContext,
    account_id: near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<Option<near_primitives::views::AccountView>> {
    for network in context.config.networks.iter() {
        loop {
            match tokio::runtime::Runtime::new().unwrap().block_on(
                crate::common::get_account_state(
                    network.1.clone(),
                    account_id.clone(),
                    near_primitives::types::Finality::Final.into(),
                ),
            ) {
                Ok(optional_account_view) => {
                    if optional_account_view.is_some() {
                        return Ok(optional_account_view);
                    } else {
                        return Ok(None);
                    }
                }
                Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(_)) => {
                    println!("\nAddress information not found: A host or server name was specified, or the network connection <{}> is missing. So now there is no way to check if <{}> exists.", network.1.network_name, account_id);

                    let choose_input = vec![
                        "Yes, I want to check the account ID again.",
                        "No, I don't want to check the account ID again.",
                    ];
                    let select_choose_input = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("Do you want to check the account ID again on this network?")
                        .items(&choose_input)
                        .default(0)
                        .interact_on_opt(&Term::stderr())?;
                    if matches!(select_choose_input, Some(1)) {
                        break;
                    }
                }
                Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(
                    near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(
                        near_jsonrpc_primitives::types::query::RpcQueryError::UnknownAccount {
                            ..
                        },
                    ),
                )) => {
                    break;
                }
                Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(_)) => {
                    println!(
                        "Unable to verify the existence of account <{}> on network <{}>",
                        account_id, network.1.network_name
                    );
                    break;
                }
            }
        }
    }
    Ok(None)
}

async fn validate_signer_account_id(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
) -> crate::CliResult {
    for retries_left in (0..5).rev() {
        match crate::common::get_account_state(
            network_config.clone(),
            account_id.clone(),
            near_primitives::types::Finality::Final.into(),
        )
        .await
        {
            Ok(optional_account_view) => {
                if optional_account_view.is_none() {
                    return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                        "\nSigner account <{}> does not yet exist on the network <{}>.",
                        account_id,
                        network_config.network_name
                    ));
                } else {
                    return Ok(());
                }
            }
            Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(err)) => {
                println!("Transport error request. If you want to exit the program, press ^C.\nThe next try to send this request is happening right now. Please wait ...");
                if retries_left == 0 {
                    return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!("{err}"));
                }
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
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
                    "Signer account {requested_account_id} does not exist now."
                ));
            }
            Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(err)) => {
                return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(err.to_string()))
            }
        }
    }
    Ok(())
}

async fn validate_new_account_id(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
) -> crate::CliResult {
    for retries_left in (0..5).rev() {
        match crate::common::get_account_state(
            network_config.clone(),
            account_id.clone(),
            near_primitives::types::Finality::Final.into(),
        )
        .await
        {
            Ok(optional_account_view) => {
                if optional_account_view.is_some() {
                    return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                        "\nAccount <{}> already exists in network <{}>.",
                        account_id,
                        network_config.network_name
                    ));
                } else {
                    return Ok(());
                }
            }
            Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(_)) => {
                println!("Transport error request. If you want to exit the program, press ^C.\nThe next try to send this request is happening right now. Please wait ...");
                if retries_left == 0 {
                    return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!("Failed to lookup address information: nodename nor servname provided, or no network connection. So now there is no way to check if <{}> exists.", account_id));
                }
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await
            }
            Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(
                near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(
                    near_jsonrpc_primitives::types::query::RpcQueryError::UnknownAccount { .. },
                ),
            )) => {
                return Ok(());
            }
            Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(_)) => {
                println!("Server error request. If you want to exit the program, press ^C.\nThe next try to send this request is happening right now. Please wait ...");
                if retries_left == 0 {
                    return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                        "Unable to verify the existence of account <{}> on network <{}>",
                        account_id,
                        network_config.network_name
                    ));
                }
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await
            }
        }
    }
    Ok(())
}
