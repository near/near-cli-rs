extern crate dirs;
use serde::Deserialize;
use std::path::PathBuf;

/// подписание сформированной транзакции с помощью файла с ключами
#[derive(Debug, Default, clap::Clap)]
pub struct CliSignKeychain {
    #[clap(subcommand)]
    submit: Option<super::sign_with_private_key::Submit>,
}

#[derive(Debug)]
pub struct SignKeychain {
    pub submit: Option<super::sign_with_private_key::Submit>,
}

impl From<CliSignKeychain> for SignKeychain {
    fn from(item: CliSignKeychain) -> Self {
        SignKeychain {
            submit: item.submit,
        }
    }
}

#[derive(Debug, Deserialize)]
struct User {
    account_id: String,
    public_key: near_crypto::PublicKey,
    private_key: near_crypto::SecretKey,
}

impl SignKeychain {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        let home_dir = dirs::home_dir().expect("Impossible to get your home dir!");
        let file_name = format!("{}.json", prepopulated_unsigned_transaction.signer_id);
        let mut path = PathBuf::from(&home_dir);
        path.push(crate::consts::DIR_NAME_KEY_CHAIN);
        path.push(file_name);
        let data = std::fs::read_to_string(path).unwrap();
        let account_json: User = serde_json::from_str(&data).unwrap();
        let sign_with_private_key = super::sign_with_private_key::SignPrivateKey {
            signer_public_key: account_json.public_key,
            signer_secret_key: account_json.private_key,
            submit: self.submit.clone(),
        };
        sign_with_private_key
            .process(prepopulated_unsigned_transaction, network_connection_config)
            .await
    }
}
