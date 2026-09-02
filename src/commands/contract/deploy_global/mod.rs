use color_eyre::eyre::Context;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

use crate::common::JsonRpcClientExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct Contract {
    #[interactive_clap(named_arg)]
    /// Specify a path to wasm file
    use_file: ContractFile,
}

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ContractFileContext)]
pub struct ContractFile {
    /// What is the file location of the contract?
    pub file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(subcommand)]
    mode: DeployGlobalMode,
}

#[derive(Debug, Clone)]
pub struct ContractFileContext {
    pub global_context: crate::GlobalContext,
    pub code: Vec<u8>,
}

impl ContractFileContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<ContractFile as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let code = std::fs::read(&scope.file_path).wrap_err_with(|| {
            format!("Failed to open or read the file: {:?}.", scope.file_path.0,)
        })?;
        Ok(Self {
            global_context: previous_context,
            code,
        })
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = ContractFileContext)]
#[interactive_clap(output_context = DeployGlobalModeContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// Choose a global contract deploy mode:
pub enum DeployGlobalMode {
    #[strum_discriminants(strum(
        message = "as-global-hash       - Deploy code as a global contract code hash (immutable, skips if already deployed, not available offline)"
    ))]
    /// Deploy code as a global contract code hash (immutable, skips if already deployed, not available offline)
    AsGlobalHash(DeployGlobalResult),
    #[strum_discriminants(strum(
        message = "as-global-account-id - Deploy code as a global contract account ID (mutable)"
    ))]
    /// Deploy code as a global contract account ID (mutable)
    AsGlobalAccountId(DeployGlobalResult),
}

#[derive(Debug, Clone)]
pub struct DeployGlobalModeContext {
    pub global_context: crate::GlobalContext,
    pub code: Vec<u8>,
    pub mode: near_primitives::action::GlobalContractDeployMode,
}

impl DeployGlobalModeContext {
    pub fn from_previous_context(
        previous_context: ContractFileContext,
        scope: &<DeployGlobalMode as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let mode = match scope {
            DeployGlobalModeDiscriminants::AsGlobalHash => {
                if previous_context.global_context.offline {
                    return Err(color_eyre::eyre::eyre!(
                        "`as-global-hash` checks the chain for existing code, so it cannot run with `--offline`. To build the transaction offline without that check, use `transaction construct-transaction ... add-action deploy-global-contract ... as-global-hash`."
                    ));
                }
                near_primitives::action::GlobalContractDeployMode::CodeHash
            }
            DeployGlobalModeDiscriminants::AsGlobalAccountId => {
                near_primitives::action::GlobalContractDeployMode::AccountId
            }
        };
        Ok(DeployGlobalModeContext {
            global_context: previous_context.global_context,
            code: previous_context.code,
            mode,
        })
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = DeployGlobalModeContext)]
#[interactive_clap(output_context = DeployGlobalResultContext)]
pub struct DeployGlobalResult {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the signer account ID?
    account_id: crate::types::account_id::AccountId,

    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

impl DeployGlobalResult {
    pub fn input_account_id(
        context: &DeployGlobalModeContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        let question = match context.mode {
            near_primitives::action::GlobalContractDeployMode::CodeHash => {
                "What is the signer account ID?"
            }
            near_primitives::action::GlobalContractDeployMode::AccountId => {
                "What is the contract account ID?"
            }
        };
        crate::common::input_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            question,
        )
    }
}

pub struct DeployGlobalResultContext {
    pub global_context: crate::GlobalContext,
    pub code: Vec<u8>,
    pub mode: near_primitives::action::GlobalContractDeployMode,
    pub account_id: near_primitives::types::AccountId,
}

impl DeployGlobalResultContext {
    pub fn from_previous_context(
        previous_context: DeployGlobalModeContext,
        scope: &<DeployGlobalResult as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            code: previous_context.code,
            mode: previous_context.mode,
            account_id: scope.account_id.clone().into(),
        })
    }
}

/// Whether global contract code hashing to `expected_hash` is already deployed.
fn global_contract_exists(
    expected_hash: near_primitives::hash::CryptoHash,
    query_result: crate::common::BoxedJsonRpcResult<
        near_jsonrpc_primitives::types::query::RpcQueryResponse,
        near_jsonrpc_primitives::types::query::RpcQueryError,
    >,
) -> color_eyre::eyre::Result<bool> {
    match query_result {
        Ok(response) => match response.kind {
            near_jsonrpc_primitives::types::query::QueryResponseKind::ViewCode(code)
                if near_primitives::hash::CryptoHash::hash_bytes(&code.code) == expected_hash =>
            {
                Ok(true)
            }
            _ => Err(color_eyre::eyre::eyre!(
                "Unexpected response when querying global contract <{expected_hash}>"
            )),
        },
        Err(err)
            if matches!(
                err.handler_error(),
                Some(
                    near_jsonrpc_primitives::types::query::RpcQueryError::NoGlobalContractCode { .. }
                )
            ) =>
        {
            Ok(false)
        }
        Err(err) => {
            Err(err).wrap_err_with(|| format!("Failed to query global contract <{expected_hash}>"))
        }
    }
}

impl From<DeployGlobalResultContext> for crate::commands::ActionContext {
    fn from(item: DeployGlobalResultContext) -> Self {
        let account_id = item.account_id.clone();
        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            std::sync::Arc::new(move |network_config| {
                let mut actions = vec![near_primitives::transaction::Action::DeployGlobalContract(
                    near_primitives::action::DeployGlobalContractAction {
                        code: item.code.clone().into(),
                        deploy_mode: item.mode.clone(),
                    },
                )];

                if item.mode == near_primitives::action::GlobalContractDeployMode::CodeHash {
                    let code_hash = near_primitives::hash::CryptoHash::hash_bytes(&item.code);
                    tracing::info!(parent: &tracing::Span::none(), "Wasm code hash: {code_hash}");
                    let query_result = network_config.json_rpc_client().blocking_call(
                        near_jsonrpc_client::methods::query::RpcQueryRequest {
                            block_reference: near_primitives::types::Finality::Final.into(),
                            request: near_primitives::views::QueryRequest::ViewGlobalContractCode {
                                code_hash,
                            },
                        },
                    );
                    if global_contract_exists(code_hash, query_result)? {
                        tracing::info!(
                            parent: &tracing::Span::none(),
                            "Global contract <{code_hash}> already exists on <{}> network. No transaction needed.",
                            network_config.network_name
                        );
                        actions.clear();
                    }
                }

                Ok(crate::commands::PrepopulatedTransaction {
                    signer_id: item.account_id.clone(),
                    receiver_id: item.account_id.clone(),
                    actions,
                })
            });

        Self {
            global_context: item.global_context,
            interacting_with_account_ids: vec![account_id],
            get_prepopulated_transaction_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepopulated_unsigned_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
            sign_as_delegate_action: false,
            on_sending_delegate_action_callback: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_jsonrpc_primitives::types::query::{
        QueryResponseKind, RpcQueryError, RpcQueryResponse,
    };
    use near_primitives::hash::CryptoHash;

    const CODE: &[u8] = b"\0asm\x01\0\0\0";

    fn handler_error(
        error: RpcQueryError,
    ) -> Box<near_jsonrpc_client::errors::JsonRpcError<RpcQueryError>> {
        Box::new(near_jsonrpc_client::errors::JsonRpcError::ServerError(
            near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(error),
        ))
    }

    fn view_code(code: &[u8]) -> RpcQueryResponse {
        RpcQueryResponse {
            kind: QueryResponseKind::ViewCode(near_primitives::views::ContractCodeView {
                code: code.to_vec(),
                hash: CryptoHash::hash_bytes(code),
            }),
            block_height: 1,
            block_hash: CryptoHash::default(),
        }
    }

    #[test]
    fn absent_code_does_not_exist() {
        let hash = CryptoHash::hash_bytes(CODE);
        let absent = handler_error(RpcQueryError::NoGlobalContractCode {
            identifier: near_primitives::action::GlobalContractIdentifier::CodeHash(hash),
            block_height: 1,
            block_hash: CryptoHash::default(),
        });
        assert!(!global_contract_exists(hash, Err(absent)).unwrap());
    }

    #[test]
    fn matching_code_exists() {
        let hash = CryptoHash::hash_bytes(CODE);
        assert!(global_contract_exists(hash, Ok(view_code(CODE))).unwrap());
    }

    #[test]
    fn mismatching_code_is_an_error() {
        let hash = CryptoHash::hash_bytes(CODE);
        assert!(global_contract_exists(hash, Ok(view_code(b"other"))).is_err());
    }

    #[test]
    fn other_rpc_errors_are_errors() {
        let hash = CryptoHash::hash_bytes(CODE);
        let err = handler_error(RpcQueryError::NoSyncedBlocks);
        assert!(global_contract_exists(hash, Err(err)).is_err());
    }
}
