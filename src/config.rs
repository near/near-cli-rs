#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub credentials_home_dir: std::path::PathBuf,
    pub networks: linked_hash_map::LinkedHashMap<String, NetworkConfig>,
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
}

impl Default for Config {
    fn default() -> Self {
        let home_dir = dirs::home_dir().expect("Impossible to get your home dir!");
        let mut credentials_home_dir = std::path::PathBuf::from(&home_dir);
        credentials_home_dir.push(".near-credentials");

        let mut networks = linked_hash_map::LinkedHashMap::new();
        networks.insert(
            "mainnet".to_string(),
            NetworkConfig {
                network_name: "mainnet".to_string(),
                rpc_url: "https://archival-rpc.mainnet.near.org".parse().unwrap(),
                wallet_url: "https://wallet.mainnet.near.org".parse().unwrap(),
                explorer_transaction_url: "https://explorer.mainnet.near.org/transactions/"
                    .parse()
                    .unwrap(),
                rpc_api_key: None,
                linkdrop_account_id: Some("near".parse().unwrap()),
            },
        );
        networks.insert(
            "testnet".to_string(),
            NetworkConfig {
                network_name: "testnet".to_string(),
                rpc_url: "https://archival-rpc.testnet.near.org".parse().unwrap(),
                wallet_url: "https://wallet.testnet.near.org".parse().unwrap(),
                explorer_transaction_url: "https://explorer.testnet.near.org/transactions/"
                    .parse()
                    .unwrap(),
                rpc_api_key: None,
                linkdrop_account_id: Some("testnet".parse().unwrap()),
            },
        );
        networks.insert(
            "shardnet".to_string(),
            NetworkConfig {
                network_name: "shardnet".to_string(),
                rpc_url: "https://rpc.shardnet.near.org".parse().unwrap(),
                wallet_url: "https://wallet.shardnet.near.org".parse().unwrap(),
                explorer_transaction_url: "https://explorer.shardnet.near.org/transactions/"
                    .parse()
                    .unwrap(),
                rpc_api_key: None,
                linkdrop_account_id: Some("shardnet".parse().unwrap()),
            },
        );
        Self {
            credentials_home_dir,
            networks,
        }
    }
}

impl NetworkConfig {
    pub fn json_rpc_client(&self) -> near_jsonrpc_client::JsonRpcClient {
        let mut json_rpc_client = near_jsonrpc_client::JsonRpcClient::connect(self.rpc_url.clone());
        if let Some(rpc_api_key) = self.rpc_api_key.clone() {
            json_rpc_client = json_rpc_client.header(rpc_api_key.0)
        };
        json_rpc_client
    }
}
