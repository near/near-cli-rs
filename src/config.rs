use color_eyre::eyre::WrapErr;
use std::str::FromStr;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub credentials_home_dir: std::path::PathBuf,
    pub network_connection: linked_hash_map::LinkedHashMap<String, NetworkConfig>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkConfig {
    pub network_name: String,
    pub rpc_url: url::Url,
    pub rpc_api_key: Option<crate::types::api_key::ApiKey>,
    pub wallet_url: url::Url,
    pub explorer_transaction_url: url::Url,
    // https://github.com/near/near-cli-rs/issues/116
    pub linkdrop_account_id: Option<near_primitives::types::AccountId>,
    // https://docs.near.org/social/contract
    pub near_social_db_contract_account_id: Option<near_primitives::types::AccountId>,
    pub faucet_url: Option<url::Url>,
    pub meta_transaction_relayer_url: Option<url::Url>,
}

impl Default for Config {
    fn default() -> Self {
        let home_dir = dirs::home_dir().expect("Impossible to get your home dir!");
        let mut credentials_home_dir = std::path::PathBuf::from(&home_dir);
        credentials_home_dir.push(".near-credentials");

        let mut network_connection = linked_hash_map::LinkedHashMap::new();
        network_connection.insert(
            "mainnet".to_string(),
            NetworkConfig {
                network_name: "mainnet".to_string(),
                rpc_url: "https://archival-rpc.mainnet.near.org".parse().unwrap(),
                wallet_url: "https://app.mynearwallet.com/".parse().unwrap(),
                explorer_transaction_url: "https://explorer.near.org/transactions/"
                    .parse()
                    .unwrap(),
                rpc_api_key: None,
                linkdrop_account_id: Some("near".parse().unwrap()),
                near_social_db_contract_account_id: Some("social.near".parse().unwrap()),
                faucet_url: None,
                meta_transaction_relayer_url: None,
            },
        );
        network_connection.insert(
            "testnet".to_string(),
            NetworkConfig {
                network_name: "testnet".to_string(),
                rpc_url: "https://archival-rpc.testnet.near.org".parse().unwrap(),
                wallet_url: "https://testnet.mynearwallet.com/".parse().unwrap(),
                explorer_transaction_url: "https://explorer.testnet.near.org/transactions/"
                    .parse()
                    .unwrap(),
                rpc_api_key: None,
                linkdrop_account_id: Some("testnet".parse().unwrap()),
                near_social_db_contract_account_id: Some("v1.social08.testnet".parse().unwrap()),
                faucet_url: Some("https://helper.nearprotocol.com/account".parse().unwrap()),
                meta_transaction_relayer_url: None,
            },
        );
        Self {
            credentials_home_dir,
            network_connection,
        }
    }
}

impl NetworkConfig {
    pub fn json_rpc_client(&self) -> near_jsonrpc_client::JsonRpcClient {
        let mut json_rpc_client =
            near_jsonrpc_client::JsonRpcClient::connect(self.rpc_url.as_ref());
        if let Some(rpc_api_key) = &self.rpc_api_key {
            json_rpc_client =
                json_rpc_client.header(near_jsonrpc_client::auth::ApiKey::from(rpc_api_key.clone()))
        };
        json_rpc_client
    }

    pub fn get_near_social_account_id_from_network(
        &self,
    ) -> color_eyre::eyre::Result<near_primitives::types::AccountId> {
        match self.network_name.as_str() {
            "mainnet" => near_primitives::types::AccountId::from_str("social.near")
                .wrap_err("Internal error"),
            "testnet" => near_primitives::types::AccountId::from_str("v1.social08.testnet")
                .wrap_err("Internal error"),
            _ => color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "This network does not provide the \"near-social\" contract"
            )),
        }
    }
}
