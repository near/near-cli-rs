mod block_hash;
mod block_height;

#[derive(clap::Parser)]
pub struct BlockIdSelector {
    #[clap(subcommand)]
    pub block_id: BlockId,
}

#[derive(clap::Subcommand)]
pub enum BlockId {
    Final(super::contract_id::CliAccountId),
    Height(block_height::BlockIdHeight),
    Hash(block_hash::BlockIdHash),
}

impl BlockId {
    pub async fn process(self, client: near_jsonrpc_client::JsonRpcClient) {
        match self {
            BlockId::Final(acc) => acc.process(
                client,
                near_primitives::types::BlockReference::Finality(Default::default()),
            ).await,
            BlockId::Height(height) => height.process(client).await,
            BlockId::Hash(_) => todo!(),
        }
    }
}
