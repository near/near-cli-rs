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
    pub url: Option<crate::common::AvailableRpcServerUrl>,
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
        let url: crate::common::AvailableRpcServerUrl = match self.url {
            Some(url) => url,
            None => Input::new()
                .with_prompt("What is the RPC endpoint?")
                .interact_text()
                .unwrap(),
        };
        Server {
            connection_config: crate::common::ConnectionConfig::Custom { url: url.inner },
        }
    }
}

impl Server {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        let generate_keypair: crate::commands::utils_command::generate_keypair_subcommand::CliGenerateKeypair =
            crate::commands::utils_command::generate_keypair_subcommand::CliGenerateKeypair::default();

        let key_pair_properties: crate::commands::utils_command::generate_keypair_subcommand::KeyPairProperties =
            generate_keypair.generate_keypair().await?;

        let url_wallet: url::Url = crate::consts::WALLET_URL.parse()?;
        let url_login: url::Url = url_wallet.join("login/?title=NEAR+CLI")?;
        let url: url::Url = url::Url::parse_with_params(
            url_login.as_str(),
            &[
                ("public_key", &key_pair_properties.public_key_str),
                ("success_url", &"http://127.0.0.1:8080".to_string()),
            ],
        )?;
        url.open();

        let public_key: near_crypto::PublicKey =
            near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?;

        let account_id = get_account_from_cli(
            public_key,
            prepopulated_unsigned_transaction,
            Some(self.connection_config),
        )
        .await?;
        if !account_id.is_empty() {
            save_account(&account_id, key_pair_properties).await?
        };
        Ok(())
    }
}

async fn get_account_from_cli(
    public_key: near_crypto::PublicKey,
    prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    network_connection_config: Option<crate::common::ConnectionConfig>,
) -> color_eyre::eyre::Result<String> {
    let account_id: String = "volodymyr.testnet".to_string();
    add_full_access_key(
        account_id.clone(),
        public_key,
        prepopulated_unsigned_transaction,
        network_connection_config,
    )
    .await?;
    Ok(account_id)
}

async fn add_full_access_key(
    account_id: String,
    public_key: near_crypto::PublicKey,
    prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    network_connection_config: Option<crate::common::ConnectionConfig>,
) -> crate::CliResult {
    let access_key: near_primitives::account::AccessKey = near_primitives::account::AccessKey {
        nonce: 0,
        permission: near_primitives::account::AccessKeyPermission::FullAccess,
    };
    let action =
        near_primitives::transaction::Action::AddKey(near_primitives::transaction::AddKeyAction {
            public_key,
            access_key,
        });
    let mut actions = prepopulated_unsigned_transaction.actions.clone();
    actions.push(action);
    let unsigned_transaction = near_primitives::transaction::Transaction {
        actions,
        signer_id: account_id.clone(),
        receiver_id: account_id.clone(),
        ..prepopulated_unsigned_transaction
    };

    let sign_with_key_chain = crate::commands::construct_transaction_command::sign_transaction::sign_with_keychain::SignKeychain {
        submit: Some(crate::commands::construct_transaction_command::sign_transaction::sign_with_private_key::Submit::Send)
    };
    sign_with_key_chain
        .process(unsigned_transaction, network_connection_config)
        .await
}

async fn save_account(
    account_id: &str,
    key_pair_properties: crate::commands::utils_command::generate_keypair_subcommand::KeyPairProperties,
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
    path.push(crate::consts::DIR_NAME_TESTNET);
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
