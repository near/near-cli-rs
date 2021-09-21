use std::str::FromStr;

use dialoguer::Input;

use near_jsonrpc_client::{
    methods::{self, EXPERIMENTAL_genesis_config},
    JsonRpcClient,
};
use near_jsonrpc_primitives::types::transactions::TransactionInfo;

/// Specify the block_id hash for this account to view
#[derive(Debug, Default, Clone, clap::Clap)]
pub struct CliBlockIdHash {
    block_id_hash: Option<near_primitives::hash::CryptoHash>,
}

#[derive(Debug, Clone)]
pub struct BlockIdHash {
    block_id_hash: near_primitives::hash::CryptoHash,
}

impl CliBlockIdHash {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = std::collections::VecDeque::new();
        if let Some(block_id_hash) = &self.block_id_hash {
            args.push_front(block_id_hash.to_string());
        }
        args
    }
}

impl From<BlockIdHash> for CliBlockIdHash {
    fn from(block_id_hash: BlockIdHash) -> Self {
        Self {
            block_id_hash: Some(block_id_hash.block_id_hash),
        }
    }
}

impl From<CliBlockIdHash> for BlockIdHash {
    fn from(item: CliBlockIdHash) -> Self {
        let block_id_hash: near_primitives::hash::CryptoHash = match item.block_id_hash {
            Some(cli_block_id_hash) => cli_block_id_hash,
            None => BlockIdHash::input_block_id_hash(),
        };
        Self { block_id_hash }
    }
}

impl BlockIdHash {
    pub fn input_block_id_hash() -> near_primitives::hash::CryptoHash {
        Input::new()
            .with_prompt("Type the block ID hash")
            .interact_text()
            .unwrap()
    }

    fn rpc_client(&self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::JsonRpcClient::new().connect(&selected_server_url)
    }

    pub async fn process(
        self,
        account_id: near_primitives::types::AccountId,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        self.display_validators_info(
            near_primitives::types::EpochReference::Latest,
            &network_connection_config,
        )
        .await?;
        Ok(())
    }

    async fn display_validators_info(
        &self,
        epoch: near_primitives::types::EpochReference,
        network_connection_config: &crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        let client = JsonRpcClient::new().connect(network_connection_config.rpc_url().as_str());

        let genesis_config_request = methods::EXPERIMENTAL_genesis_config::RpcGenesisConfigRequest;

        let genesis_config = client.clone().call(&genesis_config_request).await.unwrap();

        // //TODO: make it pretty
        println!("{:?}", genesis_config);
        println!("------------------------------------");

        let validators_request = methods::validators::RpcValidatorRequest {
            epoch_reference: epoch,
        };

        let validator_info = client.call(&validators_request).await.unwrap();

        //TODO: make it pretty
        println!("{:?}", validator_info);

        Ok(())
    }
}
