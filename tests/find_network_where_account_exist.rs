use near_cli_rs::common::find_network_where_account_exist;
use near_cli_rs::config::{Config, NetworkConfig};
use near_cli_rs::{GlobalContext, Verbosity};

fn create_test_context_with_default() -> GlobalContext {
    let config = Config::default();
    GlobalContext {
        config,
        offline: false,
        verbosity: Verbosity::Interactive,
    }
}

fn create_test_context_with_empty_network_connection() -> GlobalContext {
    let config = Config {
        network_connection: linked_hash_map::LinkedHashMap::new(),
        credentials_home_dir: std::env::home_dir().expect("Impossible to get your home dir!"),
    };
    GlobalContext {
        config,
        offline: false,
        verbosity: Verbosity::Interactive,
    }
}

fn create_test_context_with_failed_rpc_on_testnet() -> GlobalContext {
    let mut network_connection = linked_hash_map::LinkedHashMap::new();
    network_connection.insert(
        "mainnet".to_string(),
        NetworkConfig {
            network_name: "mainnet".to_string(),
            rpc_url: "https://archival-rpc.mainnet.fastnear.com/"
                .parse()
                .unwrap(),
            wallet_url: "https://app.mynearwallet.com/".parse().unwrap(),
            explorer_transaction_url: "https://explorer.near.org/transactions/".parse().unwrap(),
            rpc_api_key: None,
            linkdrop_account_id: Some("near".parse().unwrap()),
            near_social_db_contract_account_id: Some("social.near".parse().unwrap()),
            faucet_url: None,
            meta_transaction_relayer_url: None,
            fastnear_url: Some("https://api.fastnear.com/".parse().unwrap()),
            staking_pools_factory_account_id: Some("poolv1.near".parse().unwrap()),
            coingecko_url: Some("https://api.coingecko.com/".parse().unwrap()),
            mpc_contract_account_id: Some("v1.signer".parse().unwrap()),
            tx_wait_until: None,
        },
    );
    network_connection.insert(
        "mainnet-fastnear".to_string(),
        NetworkConfig {
            network_name: "mainnet".to_string(),
            rpc_url: "https://rpc.mainnet.fastnear.com/".parse().unwrap(),
            rpc_api_key: None,
            wallet_url: "https://app.mynearwallet.com/".parse().unwrap(),
            explorer_transaction_url: "https://explorer.near.org/transactions/".parse().unwrap(),
            linkdrop_account_id: Some("near".parse().unwrap()),
            near_social_db_contract_account_id: Some("social.near".parse().unwrap()),
            faucet_url: None,
            meta_transaction_relayer_url: None,
            fastnear_url: Some("https://api.fastnear.com/".parse().unwrap()),
            staking_pools_factory_account_id: Some("poolv1.near".parse().unwrap()),
            coingecko_url: Some("https://api.coingecko.com/".parse().unwrap()),
            mpc_contract_account_id: Some("v1.signer".parse().unwrap()),
            tx_wait_until: None,
        },
    );
    network_connection.insert(
        "mainnet-lava".to_string(),
        NetworkConfig {
            network_name: "mainnet".to_string(),
            rpc_url: "https://near.lava.build/".parse().unwrap(),
            rpc_api_key: None,
            wallet_url: "https://app.mynearwallet.com/".parse().unwrap(),
            explorer_transaction_url: "https://explorer.near.org/transactions/".parse().unwrap(),
            linkdrop_account_id: Some("near".parse().unwrap()),
            near_social_db_contract_account_id: Some("social.near".parse().unwrap()),
            faucet_url: None,
            meta_transaction_relayer_url: None,
            fastnear_url: Some("https://api.fastnear.com/".parse().unwrap()),
            staking_pools_factory_account_id: Some("poolv1.near".parse().unwrap()),
            coingecko_url: Some("https://api.coingecko.com/".parse().unwrap()),
            mpc_contract_account_id: Some("v1.signer".parse().unwrap()),
            tx_wait_until: None,
        },
    );
    network_connection.insert(
        "testnet".to_string(),
        NetworkConfig {
            network_name: "testnet".to_string(),
            rpc_url: "https://xxx-archival-rpc.testnet.fastnear.com/"
                .parse()
                .unwrap(),
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
            mpc_contract_account_id: Some("v1.signer-prod.testnet".parse().unwrap()),
            tx_wait_until: None,
        },
    );

    let config = Config {
        network_connection,
        credentials_home_dir: std::env::home_dir().expect("Impossible to get your home dir!"),
    };
    GlobalContext {
        config,
        offline: false,
        verbosity: Verbosity::Interactive,
    }
}

fn create_test_context_with_failed_rpc_on_mainnet() -> GlobalContext {
    let mut network_connection = linked_hash_map::LinkedHashMap::new();
    network_connection.insert(
        "mainnet".to_string(),
        NetworkConfig {
            network_name: "mainnet".to_string(),
            rpc_url: "https://xxx-archival-rpc.mainnet.fastnear.com/"
                .parse()
                .unwrap(),
            wallet_url: "https://app.mynearwallet.com/".parse().unwrap(),
            explorer_transaction_url: "https://explorer.near.org/transactions/".parse().unwrap(),
            rpc_api_key: None,
            linkdrop_account_id: Some("near".parse().unwrap()),
            near_social_db_contract_account_id: Some("social.near".parse().unwrap()),
            faucet_url: None,
            meta_transaction_relayer_url: None,
            fastnear_url: Some("https://api.fastnear.com/".parse().unwrap()),
            staking_pools_factory_account_id: Some("poolv1.near".parse().unwrap()),
            coingecko_url: Some("https://api.coingecko.com/".parse().unwrap()),
            mpc_contract_account_id: Some("v1.signer".parse().unwrap()),
            tx_wait_until: None,
        },
    );
    network_connection.insert(
        "testnet".to_string(),
        NetworkConfig {
            network_name: "testnet".to_string(),
            rpc_url: "https://archival-rpc.testnet.fastnear.com/"
                .parse()
                .unwrap(),
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
            mpc_contract_account_id: Some("v1.signer-prod.testnet".parse().unwrap()),
            tx_wait_until: None,
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
            mpc_contract_account_id: Some("v1.signer-prod.testnet".parse().unwrap()),
            tx_wait_until: None,
        },
    );
    network_connection.insert(
        "testnet-lava".to_string(),
        NetworkConfig {
            network_name: "testnet".to_string(),
            rpc_url: "https://neart.lava.build/".parse().unwrap(),
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
            mpc_contract_account_id: Some("v1.signer-prod.testnet".parse().unwrap()),
            tx_wait_until: None,
        },
    );

    let config = Config {
        network_connection,
        credentials_home_dir: std::env::home_dir().expect("Impossible to get your home dir!"),
    };
    GlobalContext {
        config,
        offline: false,
        verbosity: Verbosity::Interactive,
    }
}

fn create_test_context_with_failed_rpc() -> GlobalContext {
    let mut network_connection = linked_hash_map::LinkedHashMap::new();
    network_connection.insert(
        "mainnet".to_string(),
        NetworkConfig {
            network_name: "mainnet".to_string(),
            rpc_url: "https://xxx-archival-rpc.mainnet.fastnear.com/"
                .parse()
                .unwrap(),
            wallet_url: "https://app.mynearwallet.com/".parse().unwrap(),
            explorer_transaction_url: "https://explorer.near.org/transactions/".parse().unwrap(),
            rpc_api_key: None,
            linkdrop_account_id: Some("near".parse().unwrap()),
            near_social_db_contract_account_id: Some("social.near".parse().unwrap()),
            faucet_url: None,
            meta_transaction_relayer_url: None,
            fastnear_url: Some("https://api.fastnear.com/".parse().unwrap()),
            staking_pools_factory_account_id: Some("poolv1.near".parse().unwrap()),
            coingecko_url: Some("https://api.coingecko.com/".parse().unwrap()),
            mpc_contract_account_id: Some("v1.signer".parse().unwrap()),
            tx_wait_until: None,
        },
    );
    network_connection.insert(
        "testnet".to_string(),
        NetworkConfig {
            network_name: "testnet".to_string(),
            rpc_url: "https://xxx-archival-rpc.testnet.fastnear.com/"
                .parse()
                .unwrap(),
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
            mpc_contract_account_id: Some("v1.signer-prod.testnet".parse().unwrap()),
            tx_wait_until: None,
        },
    );

    let config = Config {
        network_connection,
        credentials_home_dir: std::env::home_dir().expect("Impossible to get your home dir!"),
    };
    GlobalContext {
        config,
        offline: false,
        verbosity: Verbosity::Interactive,
    }
}

#[test]
fn test_find_network_account_exists_with_empty_network_connection() {
    // Test: Search for an account that exists on all networks with an empty network connection
    let context = create_test_context_with_empty_network_connection();

    // Expected result: Error, because there are no networks in configuration, so it's impossible to be sure that account does not exist on all networks
    let existent_account_id: near_kit::AccountId = "test.near".parse().unwrap();
    let result = find_network_where_account_exist(&context, existent_account_id.clone());
    assert!(result.is_err());
}

#[test]
fn test_find_network_account_exists_with_default_context() {
    // Test: Search for an account that exists on all networks (testnet, mainnet)
    let context = create_test_context_with_default();

    // Expected result: Returns the network that comes first in the configuration (mainnet)
    let existent_account_id: near_kit::AccountId = "test.near".parse().unwrap();
    let result = find_network_where_account_exist(&context, existent_account_id.clone());
    assert_eq!(result.unwrap().unwrap().network_name, "mainnet");
}

#[test]
#[ignore]
fn test_for_testnet_find_network_account_exists_with_default_context() {
    // Test: Search for accounts across all networks (for *.testnet)
    let context = create_test_context_with_default();

    // Expected result: Account found on the testnet
    let existent_account_id: near_kit::AccountId = "volodymyr.testnet".parse().unwrap();
    let result = find_network_where_account_exist(&context, existent_account_id.clone());
    assert_eq!(result.unwrap().unwrap().network_name, "testnet");

    // Expected result: Account does not exist on the testnet
    let non_existent_account_id: near_kit::AccountId =
        "nonexistent.volodymyr.testnet".parse().unwrap();
    let result = find_network_where_account_exist(&context, non_existent_account_id.clone());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_for_mainnet_find_network_account_exists_with_default_context() {
    // Test: Search for accounts across all networks (for *.near)
    let context = create_test_context_with_default();

    // Expected result: Account found on the mainnet
    let existent_account_id: near_kit::AccountId = "devhub.near".parse().unwrap();
    let result = find_network_where_account_exist(&context, existent_account_id.clone());
    assert_eq!(result.unwrap().unwrap().network_name, "mainnet");

    // Expected result: Account does not exist on the mainnet
    let non_existent_account_id: near_kit::AccountId =
        "zzz-nonexistent-xq7k9m2p4w.near".parse().unwrap();
    let result = find_network_where_account_exist(&context, non_existent_account_id.clone());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_find_network_account_exists_with_context_with_failed_rpc_on_testnet() {
    // Test: Search for an account that exists on all networks (testnet, mainnet) with failed RPC on testnet
    let context = create_test_context_with_failed_rpc_on_testnet();

    // Expected result: Returns mainnet, because testnet RPC is failed
    let existent_account_id: near_kit::AccountId = "test.near".parse().unwrap();
    let result = find_network_where_account_exist(&context, existent_account_id.clone());
    assert_eq!(result.unwrap().unwrap().network_name, "mainnet");
}

#[test]
#[ignore]
fn test_for_mainnet_find_network_account_exists_with_context_with_failed_rpc_on_testnet() {
    // Test: Search for accounts across all networks (for *.near) with failed RPC on testnet
    let context = create_test_context_with_failed_rpc_on_testnet();

    // Expected result: Account found on the mainnet
    let existent_account_id: near_kit::AccountId = "devhub.near".parse().unwrap();
    let result = find_network_where_account_exist(&context, existent_account_id.clone());
    assert_eq!(result.unwrap().unwrap().network_name, "mainnet");

    // Expected result: Error, because testnet RPC is failed, so it's impossible to be sure that account does not exist on the testnet
    let non_existent_account_id: near_kit::AccountId =
        "zzz-nonexistent-xq7k9m2p4w.near".parse().unwrap();
    let result = find_network_where_account_exist(&context, non_existent_account_id.clone());
    assert!(result.is_err());
}

#[test]
#[ignore]
fn test_for_testnet_find_network_account_exists_with_context_with_failed_rpc_on_testnet() {
    // Test: Search for accounts across all networks (for *.testnet) with failed RPC on testnet
    let context = create_test_context_with_failed_rpc_on_testnet();

    // Expected result: Error, because testnet RPC is failed, so it's impossible to be sure that account exists on the testnet
    let existent_account_id: near_kit::AccountId = "volodymyr.testnet".parse().unwrap();
    let result = find_network_where_account_exist(&context, existent_account_id.clone());
    assert!(result.is_err());

    // Expected result: Error, because testnet RPC is failed, so it's impossible to be sure that account does not exist on the testnet
    let non_existent_account_id: near_kit::AccountId =
        "nonexistent.volodymyr.testnet".parse().unwrap();
    let result = find_network_where_account_exist(&context, non_existent_account_id.clone());
    assert!(result.is_err());
}

#[test]
#[ignore]
fn test_find_network_account_exists_with_context_with_failed_rpc_on_mainnet() {
    // Test: Search for an account that exists on all networks (testnet, mainnet) with failed RPC on mainnet
    let context = create_test_context_with_failed_rpc_on_mainnet();

    // Expected result: Returns testnet, because mainnet RPC is failed
    let existent_account_id: near_kit::AccountId = "test.near".parse().unwrap();
    let result = find_network_where_account_exist(&context, existent_account_id.clone());
    assert_eq!(result.unwrap().unwrap().network_name, "testnet");
}

#[test]
#[ignore]
fn test_for_mainnet_find_network_account_exists_with_context_with_failed_rpc_on_mainnet() {
    // Test: Search for accounts across all networks (for *.near) with failed RPC on mainnet
    let context = create_test_context_with_failed_rpc_on_mainnet();

    // Expected result: Error, because mainnet RPC is failed, so it's impossible to be sure that account exists on the mainnet
    let existent_account_id: near_kit::AccountId = "devhub.near".parse().unwrap();
    let result = find_network_where_account_exist(&context, existent_account_id.clone());
    assert!(result.is_err());

    // Expected result: Error, because mainnet RPC is failed, so it's impossible to be sure that account does not exist on the mainnet
    let non_existent_account_id: near_kit::AccountId =
        "zzz-nonexistent-xq7k9m2p4w.near".parse().unwrap();
    let result = find_network_where_account_exist(&context, non_existent_account_id.clone());
    assert!(result.is_err());
}

#[test]
#[ignore]
fn test_for_testnet_find_network_account_exists_with_context_with_failed_rpc_on_mainnet() {
    // Test: Search for accounts across all networks (for *.testnet) with failed RPC on mainnet
    let context = create_test_context_with_failed_rpc_on_mainnet();

    // Expected result: Account found on the testnet
    let existent_account_id: near_kit::AccountId = "volodymyr.testnet".parse().unwrap();
    let result = find_network_where_account_exist(&context, existent_account_id.clone());
    assert_eq!(result.unwrap().unwrap().network_name, "testnet");

    // Expected result: Error, because mainnet RPC is failed, so it's impossible to be sure that account does not exist on the mainnet
    let non_existent_account_id: near_kit::AccountId =
        "nonexistent.volodymyr.testnet".parse().unwrap();
    let result = find_network_where_account_exist(&context, non_existent_account_id.clone());
    assert!(result.is_err());
}

#[test]
#[ignore]
fn test_find_network_account_exists_with_context_with_failed_rpc() {
    // Test: Search for an account that exists on all networks (testnet, mainnet) with failed RPC on all networks
    let context = create_test_context_with_failed_rpc();

    // Expected result: Error, because RPC is failed on all networks
    let existent_account_id: near_kit::AccountId = "test.near".parse().unwrap();
    let result = find_network_where_account_exist(&context, existent_account_id.clone());
    assert!(result.is_err());
}

#[test]
#[ignore]
fn test_for_testnet_find_network_account_exists_with_context_with_failed_rpc() {
    // Test: Search for accounts across all networks (for *.testnet) with failed RPC on all networks
    let context = create_test_context_with_failed_rpc();

    // Expected result: Error, because RPC is failed on all networks
    let existent_account_id_testnet: near_kit::AccountId = "volodymyr.testnet".parse().unwrap();
    let result = find_network_where_account_exist(&context, existent_account_id_testnet.clone());
    assert!(result.is_err());

    // Expected result: Error, because RPC is failed on all networks
    let non_existent_account_id_testnet: near_kit::AccountId =
        "nonexistent.volodymyr.testnet".parse().unwrap();
    let result =
        find_network_where_account_exist(&context, non_existent_account_id_testnet.clone());
    assert!(result.is_err());
}

#[test]
#[ignore]
fn test_for_mainnet_find_network_account_exists_with_context_with_failed_rpc() {
    // Test: Search for accounts across all networks (for *.near) with failed RPC on all networks
    let context = create_test_context_with_failed_rpc();

    // Expected result: Error, because RPC is failed on all networks
    let existent_account_id_mainnet: near_kit::AccountId = "devhub.near".parse().unwrap();
    let result = find_network_where_account_exist(&context, existent_account_id_mainnet.clone());
    assert!(result.is_err());

    // Expected result: Error, because RPC is failed on all networks
    let non_existent_account_id_mainnet: near_kit::AccountId =
        "zzz-nonexistent-xq7k9m2p4w.near".parse().unwrap();
    let result =
        find_network_where_account_exist(&context, non_existent_account_id_mainnet.clone());
    assert!(result.is_err());
}
