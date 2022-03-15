pub async fn online_result(
    client: near_jsonrpc_client::JsonRpcClient,
    block_reference: near_primitives::types::BlockReference,
    contract_id: near_primitives::types::AccountId,
) {
    let request = near_jsonrpc_client::methods::query::RpcQueryRequest {
        block_reference,
        request: near_primitives::views::QueryRequest::ViewCode {
            account_id: contract_id,
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
    for function in wasmer::Module::from_binary(&wasmer::Store::default(), &call_access_view.code)
        .unwrap()
        .exports()
        .filter(|e| matches!(e.ty(), wasmer::ExternType::Function(_fty)))
    {
        println!("{}", function.name());
    }
}
