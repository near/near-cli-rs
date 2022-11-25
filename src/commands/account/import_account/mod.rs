use dialoguer::{console::Term, theme::ColorfulTheme, Input, Select};
use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct Login {
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network::Network,
}

impl Login {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        let network_config = self.network_config.get_network_config(config.clone());
        login(network_config, config.credentials_home_dir).await
    }
}

async fn login(
    network_config: crate::config::NetworkConfig,
    credentials_home_dir: std::path::PathBuf,
) -> crate::CliResult {
    let key_pair_properties: crate::common::KeyPairProperties =
        crate::common::generate_keypair().await?;
    let mut url: url::Url = network_config.wallet_url.join("login/")?;
    url.query_pairs_mut()
        .append_pair("title", "NEAR CLI")
        .append_pair("public_key", &key_pair_properties.public_key_str);
    // Use `success_url` once capture mode is implemented
    //.append_pair("success_url", "http://127.0.0.1:8080");
    println!(
        "If your browser doesn't automatically open, please visit this URL:\n {}\n",
        &url.as_str()
    );
    // url.open();
    open::that(url.as_ref()).ok();

    let public_key: near_crypto::PublicKey =
        near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?;

    let account_id = loop {
        let account_id_from_cli = input_account_id()?;
        println!();
        if verify_account_id(
            account_id_from_cli.clone(),
            public_key.clone(),
            network_config.clone(),
        )
        .await?
        .is_none()
        {
            println!("\nIt is currently not possible to verify the account access key.\nYou may not be logged in to {} or you may have entered an incorrect account_id.\nYou have the option to reconfirm your account or save your access key information.\n", &url.as_str());
            let choose_input = vec![
                "Yes, I want to re-enter the account_id.",
                "No, I want to save the access key information.",
            ];
            let select_choose_input = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Would you like to re-enter the account_id?")
                .items(&choose_input)
                .default(0)
                .interact_on_opt(&Term::stderr())?;
            if matches!(select_choose_input, Some(1)) {
                break account_id_from_cli;
            }
        } else {
            break account_id_from_cli;
        }
    };
    save_access_key(
        account_id,
        key_pair_properties,
        network_config,
        credentials_home_dir,
    )?;

    Ok(())
}

fn input_account_id() -> color_eyre::eyre::Result<near_primitives::types::AccountId> {
    Ok(Input::new()
        .with_prompt("Enter account ID")
        .interact_text()?)
}

async fn verify_account_id(
    account_id: near_primitives::types::AccountId,
    public_key: near_crypto::PublicKey,
    network_config: crate::config::NetworkConfig,
) -> color_eyre::eyre::Result<Option<near_primitives::views::AccessKeyView>> {
    let _is_access_key = loop {
        match network_config
            .json_rpc_client()
            .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::ViewAccessKey {
                    account_id: account_id.clone(),
                    public_key: public_key.clone(),
                },
            })
            .await
        {
            Ok(rpc_query_response) => {
                if let near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKey(result) =
                    rpc_query_response.kind
                {
                    return Ok(Some(result));
                } else {
                    return Ok(None);
                }
            }
            Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(_)) => {
                println!("\nAddress information not found: A host or server name was specified, or the network connection <{}> is missing. So now there is no way to check if <{}> exists.", network_config.network_name, account_id);

                let choose_input = vec![
                    "Yes, I want to check the account_id again.",
                    "No, I don't want to check the account_id again.",
                ];
                let select_choose_input = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Do you want to check the account_id again on this network?")
                    .items(&choose_input)
                    .default(0)
                    .interact_on_opt(&Term::stderr())?;
                if matches!(select_choose_input, Some(1)) {
                    break false;
                }
            }
            Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(
                near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(
                    near_jsonrpc_primitives::types::query::RpcQueryError::UnknownAccount { .. },
                ),
            )) => {
                break false;
            }
            Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(_)) => {
                println!(
                    "Unable to verify the existence of account <{}> on network <{}>",
                    account_id, network_config.network_name
                );
                break false;
            }
        }
    };
    Ok(None)
}

fn save_access_key(
    account_id: near_primitives::types::AccountId,
    key_pair_properties: crate::common::KeyPairProperties,
    network_config: crate::config::NetworkConfig,
    credentials_home_dir: std::path::PathBuf,
) -> crate::CliResult {
    #[cfg(target_os = "macos")]
    {
        let items = vec![
            "Store the access key in my macOS keychain",
            "Store the access key in my legacy keychain (compatible with the old near CLI)",
        ];
        let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Select a keychain to save the access key to:")
            .items(&items)
            .default(0)
            .interact()?;
        if selection == 0 {
            let storage_message = crate::common::save_access_key_to_macos_keychain(
                network_config,
                key_pair_properties,
                &account_id,
            )
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to save the access key to the keychain: {}",
                    err
                ))
            })?;
            println!("{}", storage_message);
            return Ok(());
        }
    }
    let storage_message = crate::common::save_access_key_to_keychain(
        network_config,
        credentials_home_dir,
        key_pair_properties,
        &account_id,
    )
    .map_err(|err| {
        color_eyre::Report::msg(format!("Failed to save a file with access key: {}", err))
    })?;
    println!("{}", storage_message);
    Ok(())
}
