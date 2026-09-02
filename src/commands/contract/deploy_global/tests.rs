use super::*;

const CODE: &[u8] = b"\0asm\x01\0\0\0";

fn rpc_error(error: RpcQueryError) -> GlobalContractQueryResult {
    Err(Box::new(JsonRpcError::ServerError(
        JsonRpcServerError::HandlerError(error),
    )))
}

#[test]
fn absent_code_creates_hash_deployment_action() {
    let hash = CryptoHash::hash_bytes(CODE);
    let actions = code_hash_actions(
        CODE,
        hash,
        rpc_error(RpcQueryError::NoGlobalContractCode {
            identifier: GlobalContractIdentifier::CodeHash(hash),
            block_height: 1,
            block_hash: CryptoHash::default(),
        }),
    )
    .unwrap();

    assert!(matches!(
        actions.as_slice(),
        [Action::DeployGlobalContract(action)]
            if action.deploy_mode == GlobalContractDeployMode::CodeHash
    ));
}

#[test]
fn existing_code_returns_successful_empty_actions() {
    let hash = CryptoHash::hash_bytes(CODE);
    let response = Ok(RpcQueryResponse {
        kind: QueryResponseKind::ViewCode(near_primitives::views::ContractCodeView {
            code: CODE.to_vec(),
            hash,
        }),
        block_height: 1,
        block_hash: CryptoHash::default(),
    });

    assert!(code_hash_actions(CODE, hash, response).unwrap().is_empty());
}

#[test]
fn non_absence_rpc_error_is_a_hard_failure() {
    assert!(
        code_hash_actions(
            CODE,
            CryptoHash::hash_bytes(CODE),
            rpc_error(RpcQueryError::NoSyncedBlocks),
        )
        .is_err()
    );
}
