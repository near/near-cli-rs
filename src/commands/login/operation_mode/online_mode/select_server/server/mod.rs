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
                generate_keypair.master_seed_phrase.as_deref(),
                generate_keypair.new_master_seed_phrase_words_count,
                generate_keypair.seed_phrase_hd_path,
            )
            .await?;

        let public_key: near_crypto::PublicKey =
            near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?;

        let mut url: url::Url = self.connection_config.wallet_url().join("login/")?;
        url.query_pairs_mut()
            .append_pair("title", "NEAR CLI")
            .append_pair("public_key", &key_pair_properties.public_key_str)
            .append_pair("success_url", "http://127.0.0.1:8081");
        println!(
            "If your browser doesn't automatically open, please visit this URL:\n {}\n",
            &url.as_str()
        );
        url.open();

        let account_id_from_cli = get_account_id_from_cli(&public_key, &self.connection_config);
        let account_id_from_web = get_account_id_from_web(&public_key, &self.connection_config);

        let account_id = tokio::select! {
            account_id_result = account_id_from_cli => account_id_result?,
            account_id_result = account_id_from_web => {
                let account_id = account_id_result?;
                println!("{}", account_id);
                account_id
            }
        };
        save_account(&account_id, key_pair_properties, self.connection_config).await?;
        Ok(())
    }
}

async fn get_account_id_from_web(
    public_key: &near_crypto::PublicKey,
    network_connection_config: &crate::common::ConnectionConfig,
) -> color_eyre::eyre::Result<String> {
    use std::sync::Mutex;

    use actix_web::{web, App, HttpResponse, HttpServer, Responder};
    use futures::{SinkExt, StreamExt};

    struct MyData {
        tx: Mutex<futures::channel::mpsc::Sender<String>>,
        public_key: near_crypto::PublicKey,
        network_connection_config: crate::common::ConnectionConfig,
    }

    #[derive(serde::Deserialize)]
    struct Info {
        account_id: String,
        public_key: near_crypto::PublicKey,
    }

    async fn index(query: web::Query<Info>, data: web::Data<MyData>) -> impl Responder {
        let query = query.into_inner();
        if query.public_key != data.public_key {
            return HttpResponse::NotFound();
        }
        if let Err(_) = verify_account_id(
            query.account_id.clone(),
            query.public_key.clone(),
            &data.network_connection_config,
        )
        .await
        {
            return HttpResponse::NotFound();
        }
        data.tx
            .lock()
            .unwrap()
            .send(query.account_id)
            .await
            .unwrap();
        HttpResponse::Ok()
    }

    let (tx, web_rx) = futures::channel::oneshot::channel();

    let public_key = public_key.to_owned();
    let network_connection_config = network_connection_config.to_owned();
    actix::spawn(async move {
        let (tx2, mut web2_rx) = futures::channel::mpsc::channel(1);
        let mut server = HttpServer::new(move || {
            let tx = Mutex::new(tx2.clone());
            App::new()
                .data(MyData {
                    tx,
                    public_key: public_key.clone(),
                    network_connection_config: network_connection_config.clone(),
                })
                .service(web::resource("/").route(web::get().to(index)))
        })
        .bind("127.0.0.1:8081")
        .unwrap()
        .run();
        tokio::select! {
            _ = &mut server => {},
            account_id = web2_rx.next() => {
                tx.send(account_id.unwrap()).unwrap();
                server.stop(true).await;
            }
        }
    });

    Ok(web_rx.await?)
}

async fn get_account_id_from_cli(
    public_key: &near_crypto::PublicKey,
    network_connection_config: &crate::common::ConnectionConfig,
) -> color_eyre::eyre::Result<String> {
    loop {
        let (input_tx, input_rx) = futures::channel::oneshot::channel();
        let input_account_id = std::thread::spawn(move || {
            let account_id = input_account_id();
            input_tx.send(account_id).unwrap();
        });

        let account_id = input_rx.await?;
        input_account_id
            .join()
            .expect("The input thread should be joinable since we already awaited for the input");
        if let Err(err) = verify_account_id(
            account_id.clone(),
            public_key.clone(),
            &network_connection_config,
        )
        .await
        {
            println!("Account {} does not have a matching public key ({}). Check that you entered the correct account ID or try again. Details: {}", account_id, public_key, err);
        } else {
            break Ok(account_id);
        }
    }
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
    network_connection_config: &crate::common::ConnectionConfig,
) -> color_eyre::eyre::Result<()> {
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
        serde_json::to_string_pretty(&serde_json::json!({
            "account_id": account_id,
            "master_seed_phrase": key_pair_properties.master_seed_phrase,
            "seed_phrase_hd_path": key_pair_properties.seed_phrase_hd_path.to_string(),
            "public_key": key_pair_properties.public_key_str,
            "private_key": key_pair_properties.secret_keypair_str,
        }))
        .unwrap()
    );
    let home_dir = dirs::home_dir().expect("Impossible to get your home dir!");
    let file_name: std::path::PathBuf = format!("{}.json", account_id).into();
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
