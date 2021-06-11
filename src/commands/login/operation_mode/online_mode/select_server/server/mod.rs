use std::io::Write;
use std::str::FromStr;

use dialoguer::Input;
use url_open::UrlOpen;

/// предустановленный RPC-сервер
#[derive(Debug, Default, clap::Clap)]
pub struct CliServer {}

/// данные для custom server
#[derive(Debug, Default, clap::Clap)]
pub struct CliCustomServer {
    #[clap(long)]
    pub url: Option<url::Url>,
}

#[derive(Debug)]
pub struct Server {
    pub connection_config: crate::common::ConnectionConfig,
}

impl CliServer {
    pub fn into_server(self, connection_config: crate::common::ConnectionConfig) -> Server {
        Server { connection_config }
    }
}

impl CliCustomServer {
    pub fn into_server(self) -> Server {
        let url: url::Url = match self.url {
            Some(url) => url,
            None => Input::new()
                .with_prompt("What is the wallet url?")
                .interact_text()
                .unwrap(),
        };
        Server {
            connection_config: crate::common::ConnectionConfig::Custom { url },
        }
    }
}

impl Server {
    pub async fn process(self) -> crate::CliResult {
        let generate_keypair: crate::commands::utils_command::generate_keypair_subcommand::CliGenerateKeypair =
            crate::commands::utils_command::generate_keypair_subcommand::CliGenerateKeypair::default();

        let key_pair_properties: crate::common::KeyPairProperties =
            crate::common::generate_keypair(
                generate_keypair.master_seed_phrase.clone(),
                generate_keypair.new_master_seed_phrase_words_count.clone(),
                generate_keypair.seed_phrase_hd_path.clone(),
            )
            .await?;

        let mut url: url::Url = self.connection_config.wallet_url().join("login/")?;
        url.query_pairs_mut()
            .append_pair("title", "NEAR+CLI")
            .append_pair("public_key", &key_pair_properties.public_key_str);
        // .append_pair("success_url", "http://127.0.0.1:8080");
        println!(
            "If your browser doesn't automatically open, please visit this URL:\n {}\n",
            &url.as_str()
        );
        url.open();

        let public_key: near_crypto::PublicKey =
            near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?;

        let account_id = get_account_from_cli(public_key, self.connection_config.clone()).await?;
        if !account_id.is_empty() {
            save_account(&account_id, key_pair_properties, self.connection_config).await?
        };
        Ok(())
    }
}

async fn get_account_from_cli(
    public_key: near_crypto::PublicKey,
    network_connection_config: crate::common::ConnectionConfig,
) -> color_eyre::eyre::Result<String> {
    let account_id = input_account_id();
    verify_account_id(account_id.clone(), public_key, network_connection_config)
        .await
        .map_err(|err| color_eyre::Report::msg(format!("Failed account ID: {:?}", err)))?;
    Ok(account_id)
}

fn input_account_id() -> String {
    Input::new()
        .with_prompt("Enter account ID")
        .interact_text()
        .unwrap()
}

fn rpc_client(selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
    near_jsonrpc_client::new_client(&selected_server_url)
}

async fn verify_account_id(
    account_id: String,
    public_key: near_crypto::PublicKey,
    network_connection_config: crate::common::ConnectionConfig,
) -> crate::CliResult {
    rpc_client(network_connection_config.rpc_url().as_str())
        .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
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

async fn save_account(
    account_id: &str,
    key_pair_properties: crate::common::KeyPairProperties,
    network_connection_config: crate::common::ConnectionConfig,
) -> crate::CliResult {
    let buf = format!(
        "{}",
        serde_json::json!({
        "account_id": account_id,
        "public_key": key_pair_properties.public_key_str.clone(),
        "private_key": key_pair_properties.secret_keypair_str.clone(),
        })
    );
    let home_dir = dirs::home_dir().expect("Impossible to get your home dir!");
    let file_name: std::path::PathBuf = format!("{}.json", &account_id).into();
    let mut path = std::path::PathBuf::from(&home_dir);
    path.push(network_connection_config.dir_name());
    std::fs::create_dir_all(&path)?;
    path.push(file_name);
    std::fs::File::create(&path)
        .map_err(|err| color_eyre::Report::msg(format!("Failed to create file: {:?}", err)))?
        .write(buf.as_bytes())
        .map_err(|err| color_eyre::Report::msg(format!("Failed to write to file: {:?}", err)))?;
    println!(
        "\n\n\nThe data for the access key is saved in a file {}",
        &path.display()
    );
    println!(
        "Logged in as [ {} ] with public key [ {} ] successfully",
        account_id, key_pair_properties.public_key_str
    );
    Ok(())
}
