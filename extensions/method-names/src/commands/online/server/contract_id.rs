#[derive(clap::Parser)]
pub struct CliAccountId {
    contract_id: near_primitives::types::AccountId,
}

impl CliAccountId {
    pub async fn process(self, client: near_jsonrpc_client::JsonRpcClient, block_reference: near_primitives::types::BlockReference) {
        crate::common::online_result(client, block_reference, self.contract_id).await
    }
}