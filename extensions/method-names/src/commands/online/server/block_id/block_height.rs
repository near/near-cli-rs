#[derive(clap::Parser)]
pub struct BlockIdHeight {
    block_id_height: near_primitives::types::BlockHeight,
    contract_id: near_primitives::types::AccountId,
}

impl BlockIdHeight {
    pub async fn process(self, client: near_jsonrpc_client::JsonRpcClient) {
        let request = near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference: near_primitives::types::BlockReference::BlockId(
                near_primitives::types::BlockId::Height(self.block_id_height),
            ),
            request: near_primitives::views::QueryRequest::ViewCode {
                account_id: self.contract_id,
            },
        };
        let status = client.call(request).await.unwrap();
        let call_access_view =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewCode(result) =
                status.kind
            {
                result
            } else {
                todo!()
            };
        for function in
            wasmer::Module::from_binary(&wasmer::Store::default(), &call_access_view.code)
                .unwrap()
                .exports()
                .filter(|e| matches!(e.ty(), wasmer::ExternType::Function(_fty)))
        {
            println!("{}", function.name());
        }
    }
}
