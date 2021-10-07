use near_jsonrpc_client::methods::EXPERIMENTAL_genesis_config::GenesisConfig;
use near_primitives::{borsh::BorshDeserialize, views::EpochValidatorInfo};

pub type CliResult = color_eyre::eyre::Result<()>;

pub enum ConnectionConfig {
    Testnet,
    Mainnet,
    Betanet,
    Custom { url: url::Url },
}

impl ConnectionConfig {
    pub fn rpc_url(&self) -> url::Url {
        match self {
            Self::Testnet => crate::consts::TESTNET_API_SERVER_URL.parse().unwrap(),
            Self::Mainnet => crate::consts::MAINNET_API_SERVER_URL.parse().unwrap(),
            Self::Betanet => crate::consts::BETANET_API_SERVER_URL.parse().unwrap(),
            Self::Custom { url } => url.clone(),
        }
    }
}

pub async fn validators_info(
    epoch: near_primitives::types::EpochReference,
    network_connection_config: &crate::common::ConnectionConfig,
) -> (GenesisConfig, EpochValidatorInfo) {
    let client =
        near_jsonrpc_client::JsonRpcClient::connect(network_connection_config.rpc_url().as_str());

    let genesis_config_request =
        near_jsonrpc_client::methods::EXPERIMENTAL_genesis_config::RpcGenesisConfigRequest;

    let genesis_config = client.clone().call(&genesis_config_request).await.unwrap();

    let validators_request = near_jsonrpc_client::methods::validators::RpcValidatorRequest {
        epoch_reference: epoch,
    };

    let validator_info = client.call(&validators_request).await.unwrap();

    (genesis_config, validator_info)
}

pub async fn display_validators_info(
    epoch: near_primitives::types::EpochReference,
    network_connection_config: &crate::common::ConnectionConfig,
) -> crate::CliResult {
    let (genesis_config, validator_info) = validators_info(epoch, network_connection_config).await;

    //TODO: make it pretty
    println!("-------------- Validators info (should be in table) ----------------------");
    println!("Genesis config: {:?}", genesis_config);
    println!("------------------------------------");
    println!("Validator info: {:?}", validator_info);

    Ok(())
}

pub async fn display_proposals_info(
    network_connection_config: &crate::common::ConnectionConfig,
) -> crate::CliResult {
    let (genesis_config, validator_info) = validators_info(
        near_primitives::types::EpochReference::Latest,
        network_connection_config,
    )
    .await;

    //TODO: make it pretty
    println!("-------------- Proposals info (should be in table) ----------------------");
    println!("Genesis config: {:?}", genesis_config);
    println!("------------------------------------");
    println!("Validator info: {:?}", validator_info);

    Ok(())
}