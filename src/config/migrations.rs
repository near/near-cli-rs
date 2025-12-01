use crate::config::Config as ConfigV4;
use crate::config::NetworkConfig as NetworkConfigV4;
use NetworkConfigV3 as NetworkConfigV2;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigV1 {
    pub credentials_home_dir: std::path::PathBuf,
    pub network_connection: linked_hash_map::LinkedHashMap<String, NetworkConfigV1>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigV2 {
    pub credentials_home_dir: std::path::PathBuf,
    pub network_connection: linked_hash_map::LinkedHashMap<String, NetworkConfigV2>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigV3 {
    pub credentials_home_dir: std::path::PathBuf,
    pub network_connection: linked_hash_map::LinkedHashMap<String, NetworkConfigV3>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkConfigV1 {
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkConfigV3 {
    pub network_name: String,
    pub rpc_url: url::Url,
    pub rpc_api_key: Option<crate::types::api_key::ApiKey>,
    pub wallet_url: url::Url,
    pub explorer_transaction_url: url::Url,
    pub linkdrop_account_id: Option<near_primitives::types::AccountId>,
    pub near_social_db_contract_account_id: Option<near_primitives::types::AccountId>,
    pub faucet_url: Option<url::Url>,
    pub meta_transaction_relayer_url: Option<url::Url>,
    pub fastnear_url: Option<url::Url>,
    pub staking_pools_factory_account_id: Option<near_primitives::types::AccountId>,
    pub coingecko_url: Option<url::Url>,
}

impl From<ConfigV1> for ConfigV2 {
    fn from(config: ConfigV1) -> Self {
        ConfigV2 {
            credentials_home_dir: config.credentials_home_dir,
            network_connection: config
                .network_connection
                .into_iter()
                .map(|(network_name, network_config)| (network_name, network_config.into()))
                .collect(),
        }
    }
}

impl From<ConfigV2> for ConfigV3 {
    fn from(config: ConfigV2) -> Self {
        ConfigV3 {
            credentials_home_dir: config.credentials_home_dir,
            network_connection: config
                .network_connection
                .into_iter()
                .map(|(network_name, mut network_config)| {
                    if network_name == "testnet" && network_config.fastnear_url.is_none() {
                        network_config.fastnear_url =
                            Some("https://test.api.fastnear.com/".parse().unwrap());
                    }
                    (network_name, network_config)
                })
                .collect(),
        }
    }
}

impl From<ConfigV3> for ConfigV4 {
    fn from(config: ConfigV3) -> Self {
        ConfigV4 {
            credentials_home_dir: config.credentials_home_dir,
            network_connection: config
                .network_connection
                .into_iter()
                .map(|(network_name, network_config)| (network_name, network_config.into()))
                .collect(),
        }
    }
}

impl From<NetworkConfigV1> for NetworkConfigV2 {
    fn from(network_config: NetworkConfigV1) -> Self {
        match network_config.network_name.as_str() {
            "mainnet" => NetworkConfigV2 {
                network_name: network_config.network_name,
                rpc_url: network_config.rpc_url,
                wallet_url: network_config.wallet_url,
                explorer_transaction_url: network_config.explorer_transaction_url,
                rpc_api_key: network_config.rpc_api_key,
                linkdrop_account_id: network_config.linkdrop_account_id,
                near_social_db_contract_account_id: network_config
                    .near_social_db_contract_account_id,
                faucet_url: network_config.faucet_url,
                meta_transaction_relayer_url: network_config.meta_transaction_relayer_url,
                fastnear_url: Some("https://api.fastnear.com".parse().unwrap()),
                staking_pools_factory_account_id: Some("poolv1.near".parse().unwrap()),
                coingecko_url: Some("https://api.coingecko.com/".parse().unwrap()),
            },
            "testnet" => NetworkConfigV2 {
                network_name: network_config.network_name,
                rpc_url: network_config.rpc_url,
                wallet_url: network_config.wallet_url,
                explorer_transaction_url: network_config.explorer_transaction_url,
                rpc_api_key: network_config.rpc_api_key,
                linkdrop_account_id: network_config.linkdrop_account_id,
                near_social_db_contract_account_id: network_config
                    .near_social_db_contract_account_id,
                faucet_url: network_config.faucet_url,
                meta_transaction_relayer_url: network_config.meta_transaction_relayer_url,
                fastnear_url: None,
                staking_pools_factory_account_id: Some("pool.f863973.m0".parse().unwrap()),
                coingecko_url: None,
            },
            _ => NetworkConfigV2 {
                network_name: network_config.network_name,
                rpc_url: network_config.rpc_url,
                wallet_url: network_config.wallet_url,
                explorer_transaction_url: network_config.explorer_transaction_url,
                rpc_api_key: network_config.rpc_api_key,
                linkdrop_account_id: network_config.linkdrop_account_id,
                near_social_db_contract_account_id: network_config
                    .near_social_db_contract_account_id,
                faucet_url: network_config.faucet_url,
                meta_transaction_relayer_url: network_config.meta_transaction_relayer_url,
                fastnear_url: None,
                staking_pools_factory_account_id: None,
                coingecko_url: None,
            },
        }
    }
}

impl From<NetworkConfigV3> for NetworkConfigV4 {
    fn from(network_config: NetworkConfigV3) -> Self {
        let mpc_contract_account_id: Option<near_primitives::types::AccountId> =
            match network_config.network_name.as_str() {
                "mainnet" => Some("v1.signer".parse().unwrap()),
                "testnet" => Some("v1.signer-prod.testnet".parse().unwrap()),
                _ => None,
            };

        NetworkConfigV4 {
            network_name: network_config.network_name,
            rpc_url: network_config.rpc_url,
            wallet_url: network_config.wallet_url,
            explorer_transaction_url: network_config.explorer_transaction_url,
            rpc_api_key: network_config.rpc_api_key,
            linkdrop_account_id: network_config.linkdrop_account_id,
            near_social_db_contract_account_id: network_config.near_social_db_contract_account_id,
            faucet_url: network_config.faucet_url,
            meta_transaction_relayer_url: network_config.meta_transaction_relayer_url,
            fastnear_url: network_config.fastnear_url,
            staking_pools_factory_account_id: network_config.staking_pools_factory_account_id,
            coingecko_url: network_config.coingecko_url,
            mpc_contract_account_id,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "version")]
pub enum ConfigVersion {
    #[serde(rename = "1")]
    V1(ConfigV1),
    #[serde(rename = "2")]
    V2(ConfigV2),
    // Adds fastnear_url to the testnet config if it's not present
    #[serde(rename = "3")]
    V3(ConfigV3),
    // Adds mpc_contract_account_id to the mainnet and testnet
    #[serde(rename = "4")]
    V4(ConfigV4),
}

impl ConfigVersion {
    pub fn is_latest_version(&self) -> bool {
        // Used match instead of matches! to compile fail if new version is added
        match self {
            ConfigVersion::V4(_) => true,
            ConfigVersion::V3(_) | ConfigVersion::V2(_) | ConfigVersion::V1(_) => false,
        }
    }
}
