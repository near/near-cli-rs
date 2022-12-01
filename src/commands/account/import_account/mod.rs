use dialoguer::Input;
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

    let account_id = get_account_from_cli(public_key, network_config.clone()).await?;

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
            crate::common::save_access_key_to_macos_keychain(
                network_config,
                key_pair_properties,
                &account_id,
            )
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to save the access key to the keychain: {}",
                    err
                ))
            })?;
            return Ok(());
        }
    }
    crate::common::save_access_key_to_keychain(
        network_config,
        credentials_home_dir,
        key_pair_properties.clone(),
        &account_id,
    )
    .await
    .map_err(|err| {
        color_eyre::Report::msg(format!("Failed to save a file with access key: {}", err))
    })?;
    Ok(())
}

async fn get_account_from_cli(
    public_key: near_crypto::PublicKey,
    network_config: crate::config::NetworkConfig,
) -> color_eyre::eyre::Result<near_primitives::types::AccountId> {
    let account_id = input_account_id()?;
    verify_account_id(account_id.clone(), public_key, network_config)
        .await
        .map_err(|err| color_eyre::Report::msg(format!("Failed account ID: {:?}", err)))?;
    Ok(account_id)
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
) -> crate::CliResult {
    network_config
        .json_rpc_client()
        .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference: near_primitives::types::Finality::Final.into(),
            request: near_primitives::views::QueryRequest::ViewAccessKey {
                account_id,
                public_key,
            },
        })
        .await
        .map_err(|err| {
            color_eyre::Report::msg(format!(
                "Failed to fetch query for view access key: {:?}",
                err
            ))
        })?;
    Ok(())
}
