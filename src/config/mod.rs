mod migrations;

pub type CliResult = color_eyre::eyre::Result<()>;

use color_eyre::eyre::{ContextCompat, WrapErr};
use std::{io::Write, str::FromStr};
use tracing_indicatif::span_ext::IndicatifSpanExt;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub credentials_home_dir: std::path::PathBuf,
    pub network_connection: linked_hash_map::LinkedHashMap<String, NetworkConfig>,
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
                rpc_url: "https://archival-rpc.mainnet.near.org/".parse().unwrap(),
                wallet_url: "https://app.mynearwallet.com/".parse().unwrap(),
                explorer_transaction_url: "https://explorer.near.org/transactions/"
                    .parse()
                    .unwrap(),
                rpc_api_key: None,
                linkdrop_account_id: Some("near".parse().unwrap()),
                near_social_db_contract_account_id: Some("social.near".parse().unwrap()),
                faucet_url: None,
                meta_transaction_relayer_url: None,
                fastnear_url: Some("https://api.fastnear.com/".parse().unwrap()),
                staking_pools_factory_account_id: Some("poolv1.near".parse().unwrap()),
                coingecko_url: Some("https://api.coingecko.com/".parse().unwrap()),
            },
        );
        network_connection.insert(
            "mainnet-fastnear".to_string(),
            NetworkConfig {
                network_name: "mainnet".to_string(),
                rpc_url: "https://free.rpc.fastnear.com/".parse().unwrap(),
                rpc_api_key: None,
                wallet_url: "https://app.mynearwallet.com/".parse().unwrap(),
                explorer_transaction_url: "https://explorer.near.org/transactions/"
                    .parse()
                    .unwrap(),
                linkdrop_account_id: Some("near".parse().unwrap()),
                near_social_db_contract_account_id: Some("social.near".parse().unwrap()),
                faucet_url: None,
                meta_transaction_relayer_url: None,
                fastnear_url: Some("https://api.fastnear.com/".parse().unwrap()),
                staking_pools_factory_account_id: Some("poolv1.near".parse().unwrap()),
                coingecko_url: Some("https://api.coingecko.com/".parse().unwrap()),
            },
        );
        network_connection.insert(
            "mainnet-lava".to_string(),
            NetworkConfig {
                network_name: "mainnet".to_string(),
                rpc_url: "https://near.lava.build:443/".parse().unwrap(),
                rpc_api_key: None,
                wallet_url: "https://app.mynearwallet.com/".parse().unwrap(),
                explorer_transaction_url: "https://explorer.near.org/transactions/"
                    .parse()
                    .unwrap(),
                linkdrop_account_id: Some("near".parse().unwrap()),
                near_social_db_contract_account_id: Some("social.near".parse().unwrap()),
                faucet_url: None,
                meta_transaction_relayer_url: None,
                fastnear_url: Some("https://api.fastnear.com/".parse().unwrap()),
                staking_pools_factory_account_id: Some("poolv1.near".parse().unwrap()),
                coingecko_url: Some("https://api.coingecko.com/".parse().unwrap()),
            },
        );

        network_connection.insert(
            "testnet".to_string(),
            NetworkConfig {
                network_name: "testnet".to_string(),
                rpc_url: "https://archival-rpc.testnet.near.org/".parse().unwrap(),
                wallet_url: "https://testnet.mynearwallet.com/".parse().unwrap(),
                explorer_transaction_url: "https://explorer.testnet.near.org/transactions/"
                    .parse()
                    .unwrap(),
                rpc_api_key: None,
                linkdrop_account_id: Some("testnet".parse().unwrap()),
                near_social_db_contract_account_id: Some("v1.social08.testnet".parse().unwrap()),
                faucet_url: Some("https://helper.nearprotocol.com/account".parse().unwrap()),
                meta_transaction_relayer_url: None,
                fastnear_url: Some("https://test.api.fastnear.com/".parse().unwrap()),
                staking_pools_factory_account_id: Some("pool.f863973.m0".parse().unwrap()),
                coingecko_url: None,
            },
        );
        network_connection.insert(
            "testnet-fastnear".to_string(),
            NetworkConfig {
                network_name: "testnet".to_string(),
                rpc_url: "https://test.rpc.fastnear.com/".parse().unwrap(),
                rpc_api_key: None,
                wallet_url: "https://testnet.mynearwallet.com/".parse().unwrap(),
                explorer_transaction_url: "https://explorer.testnet.near.org/transactions/"
                    .parse()
                    .unwrap(),
                linkdrop_account_id: Some("testnet".parse().unwrap()),
                near_social_db_contract_account_id: Some("v1.social08.testnet".parse().unwrap()),
                faucet_url: Some("https://helper.nearprotocol.com/account".parse().unwrap()),
                meta_transaction_relayer_url: None,
                fastnear_url: Some("https://test.api.fastnear.com/".parse().unwrap()),
                staking_pools_factory_account_id: Some("pool.f863973.m0".parse().unwrap()),
                coingecko_url: None,
            },
        );
        network_connection.insert(
            "testnet-lava".to_string(),
            NetworkConfig {
                network_name: "testnet".to_string(),
                rpc_url: "https://near-testnet.lava.build:433/".parse().unwrap(),
                rpc_api_key: None,
                wallet_url: "https://testnet.mynearwallet.com/".parse().unwrap(),
                explorer_transaction_url: "https://explorer.testnet.near.org/transactions/"
                    .parse()
                    .unwrap(),
                linkdrop_account_id: Some("testnet".parse().unwrap()),
                near_social_db_contract_account_id: Some("v1.social08.testnet".parse().unwrap()),
                faucet_url: Some("https://helper.nearprotocol.com/account".parse().unwrap()),
                meta_transaction_relayer_url: None,
                fastnear_url: Some("https://test.api.fastnear.com/".parse().unwrap()),
                staking_pools_factory_account_id: Some("pool.f863973.m0".parse().unwrap()),
                coingecko_url: None,
            },
        );

        Self {
            credentials_home_dir,
            network_connection,
        }
    }
}

impl Config {
    pub fn network_names(&self) -> Vec<String> {
        self.network_connection
            .iter()
            .map(|(_, network_config)| network_config.network_name.clone())
            .collect()
    }

    pub fn into_latest_version(self) -> migrations::ConfigVersion {
        migrations::ConfigVersion::V3(self)
    }

    pub fn get_config_toml() -> color_eyre::eyre::Result<Self> {
        if let Some(mut path_config_toml) = dirs::config_dir() {
            path_config_toml.extend(&["near-cli", "config.toml"]);

            if !path_config_toml.is_file() {
                Self::write_config_toml(crate::config::Config::default())?;
            };

            let config_toml = std::fs::read_to_string(&path_config_toml)?;

            let config_version = toml::from_str::<migrations::ConfigVersion>(&config_toml).or_else::<color_eyre::eyre::Report, _>(|err| {
                if let Ok(config_v1) = toml::from_str::<migrations::ConfigV1>(&config_toml) {
                    Ok(migrations::ConfigVersion::V1(config_v1))
                } else {
                    eprintln!("Warning: `near` CLI configuration file stored at {path_config_toml:?} could not be parsed due to: {err}");
                    eprintln!("Note: The default configuration printed below will be used instead:\n");
                    let default_config = crate::config::Config::default();
                    eprintln!("{}", toml::to_string(&default_config)?);
                    Ok(default_config.into_latest_version())
                }
            })?;

            let is_latest_version = config_version.is_latest_version();
            let config: Config = config_version.into();

            if !is_latest_version {
                Self::write_config_toml(config.clone())?;
            }

            Ok(config)
        } else {
            Ok(crate::config::Config::default())
        }
    }

    pub fn write_config_toml(self) -> CliResult {
        let config_toml = toml::to_string(&self.into_latest_version())?;
        let mut path_config_toml =
            dirs::config_dir().wrap_err("Impossible to get your config dir!")?;

        path_config_toml.push("near-cli");
        std::fs::create_dir_all(&path_config_toml)?;
        path_config_toml.push("config.toml");

        std::fs::File::create(&path_config_toml)
            .wrap_err_with(|| format!("Failed to create file: {path_config_toml:?}"))?
            .write(config_toml.as_bytes())
            .wrap_err_with(|| format!("Failed to write to file: {path_config_toml:?}"))?;

        eprintln!("Note: `near` CLI configuration is stored in {path_config_toml:?}");

        Ok(())
    }
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
    pub fastnear_url: Option<url::Url>,
    pub staking_pools_factory_account_id: Option<near_primitives::types::AccountId>,
    pub coingecko_url: Option<url::Url>,
}

impl NetworkConfig {
    pub(crate) fn get_fields(&self) -> color_eyre::eyre::Result<Vec<String>> {
        let network_config_value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(self)?)?;
        Ok(network_config_value
            .as_object()
            .wrap_err("Internal error")?
            .iter()
            .map(|(key, value)| format!("{key}: {value}"))
            .collect())
    }

    #[tracing::instrument(name = "Connecting to RPC", skip_all)]
    pub fn json_rpc_client(&self) -> near_jsonrpc_client::JsonRpcClient {
        tracing::Span::current().pb_set_message(self.rpc_url.as_str());
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
        if let Some(account_id) = self.near_social_db_contract_account_id.clone() {
            return Ok(account_id);
        }
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

impl From<migrations::ConfigVersion> for Config {
    fn from(mut config_version: migrations::ConfigVersion) -> Self {
        loop {
            config_version = match config_version {
                migrations::ConfigVersion::V1(config_v1) => {
                    eprintln!("Migrating config.toml from V1 to V2...");
                    migrations::ConfigVersion::V2(config_v1.into())
                }
                migrations::ConfigVersion::V2(config_v2) => {
                    eprintln!("Migrating config.toml from V2 to V3...");
                    migrations::ConfigVersion::V3(config_v2.into())
                }
                migrations::ConfigVersion::V3(config_v3) => {
                    break config_v3;
                }
            };
        }
    }
}
