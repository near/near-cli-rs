#[derive(clap::Args)]
pub struct BlockIdHash {
    block_id_hash: String, // TODO: replace it with CryptoHash
    contract_id: near_primitives::types::AccountId,
}

impl BlockIdHash {
    pub async fn process(self, client: near_jsonrpc_client::JsonRpcClient) {
        crate::common::online_result(
            client,
            near_primitives::types::BlockReference::BlockId(near_primitives::types::BlockId::Hash(
                self.block_id_hash.parse().unwrap(),
            )),
            self.contract_id,
        )
        .await;
    }
}
