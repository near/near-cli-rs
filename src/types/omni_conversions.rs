/// Conversion functions between near-primitives and our internal transaction types
use super::transactions as omni;

/// Error type for action conversion
#[derive(Debug, Clone)]
pub enum ActionConversionError {
    /// DeterministicStateInit action is not supported
    UnsupportedDeterministicStateInit,
}

impl std::fmt::Display for ActionConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionConversionError::UnsupportedDeterministicStateInit => {
                write!(f, "DeterministicStateInit action is not supported in internal transaction types")
            }
        }
    }
}

impl std::error::Error for ActionConversionError {}

/// Convert near_primitives::transaction::Action to our internal Action
pub fn near_action_to_omni(action: near_primitives::transaction::Action) -> Result<omni::Action, ActionConversionError> {
    Ok(match action {
        near_primitives::transaction::Action::CreateAccount(_) => {
            omni::Action::CreateAccount(omni::CreateAccountAction {})
        }
        near_primitives::transaction::Action::DeployContract(deploy) => {
            omni::Action::DeployContract(omni::DeployContractAction { code: deploy.code })
        }
        near_primitives::transaction::Action::FunctionCall(fc) => {
            omni::Action::FunctionCall(Box::new(omni::FunctionCallAction {
                method_name: fc.method_name,
                args: fc.args,
                gas: omni::U64(fc.gas.as_gas()),
                deposit: omni::U128(fc.deposit.as_yoctonear()),
            }))
        }
        near_primitives::transaction::Action::Transfer(transfer) => {
            omni::Action::Transfer(omni::TransferAction {
                deposit: omni::U128(transfer.deposit.as_yoctonear()),
            })
        }
        near_primitives::transaction::Action::Stake(stake) => {
            omni::Action::Stake(Box::new(omni::StakeAction {
                stake: omni::U128(stake.stake.as_yoctonear()),
                public_key: near_public_key_to_omni(stake.public_key),
            }))
        }
        near_primitives::transaction::Action::AddKey(add_key) => {
            omni::Action::AddKey(Box::new(omni::AddKeyAction {
                public_key: near_public_key_to_omni(add_key.public_key),
                access_key: near_access_key_to_omni(add_key.access_key),
            }))
        }
        near_primitives::transaction::Action::DeleteKey(delete_key) => {
            omni::Action::DeleteKey(Box::new(omni::DeleteKeyAction {
                public_key: near_public_key_to_omni(delete_key.public_key),
            }))
        }
        near_primitives::transaction::Action::DeleteAccount(delete_account) => {
            omni::Action::DeleteAccount(omni::DeleteAccountAction {
                beneficiary_id: delete_account.beneficiary_id,
            })
        }
        near_primitives::transaction::Action::Delegate(delegate) => {
            omni::Action::Delegate(Box::new(near_signed_delegate_action_to_omni(*delegate)))
        }
        near_primitives::transaction::Action::DeployGlobalContract(deploy) => {
            omni::Action::DeployGlobalContract(omni::DeployGlobalContractAction {
                code: deploy.code.to_vec(),
                deploy_mode: near_global_contract_deploy_mode_to_omni(deploy.deploy_mode),
            })
        }
        near_primitives::transaction::Action::UseGlobalContract(use_global) => {
            omni::Action::UseGlobalContract(Box::new(omni::UseGlobalContractAction {
                contract_identifier: near_global_contract_identifier_to_omni(
                    use_global.contract_identifier,
                ),
            }))
        }
        near_primitives::transaction::Action::DeterministicStateInit(_) => {
            return Err(ActionConversionError::UnsupportedDeterministicStateInit);
        }
    })
}

/// Convert omni_transaction Action to near_primitives::transaction::Action
pub fn omni_action_to_near(action: omni::Action) -> near_primitives::transaction::Action {
    match action {
        omni::Action::CreateAccount(_) => {
            near_primitives::transaction::Action::CreateAccount(
                near_primitives::transaction::CreateAccountAction {},
            )
        }
        omni::Action::DeployContract(deploy) => {
            near_primitives::transaction::Action::DeployContract(
                near_primitives::transaction::DeployContractAction { code: deploy.code },
            )
        }
        omni::Action::FunctionCall(fc) => {
            near_primitives::transaction::Action::FunctionCall(Box::new(
                near_primitives::transaction::FunctionCallAction {
                    method_name: fc.method_name,
                    args: fc.args,
                    gas: near_primitives::types::Gas::from_gas(fc.gas.0),
                    deposit: near_token::NearToken::from_yoctonear(fc.deposit.0),
                },
            ))
        }
        omni::Action::Transfer(transfer) => near_primitives::transaction::Action::Transfer(
            near_primitives::transaction::TransferAction {
                deposit: near_token::NearToken::from_yoctonear(transfer.deposit.0),
            },
        ),
        omni::Action::Stake(stake) => near_primitives::transaction::Action::Stake(Box::new(
            near_primitives::transaction::StakeAction {
                stake: near_token::NearToken::from_yoctonear(stake.stake.0),
                public_key: omni_public_key_to_near(stake.public_key),
            },
        )),
        omni::Action::AddKey(add_key) => near_primitives::transaction::Action::AddKey(Box::new(
            near_primitives::transaction::AddKeyAction {
                public_key: omni_public_key_to_near(add_key.public_key),
                access_key: omni_access_key_to_near(add_key.access_key),
            },
        )),
        omni::Action::DeleteKey(delete_key) => {
            near_primitives::transaction::Action::DeleteKey(Box::new(
                near_primitives::transaction::DeleteKeyAction {
                    public_key: omni_public_key_to_near(delete_key.public_key),
                },
            ))
        }
        omni::Action::DeleteAccount(delete_account) => {
            near_primitives::transaction::Action::DeleteAccount(
                near_primitives::transaction::DeleteAccountAction {
                    beneficiary_id: delete_account.beneficiary_id,
                },
            )
        }
        omni::Action::Delegate(delegate) => near_primitives::transaction::Action::Delegate(
            Box::new(omni_signed_delegate_action_to_near(*delegate)),
        ),
        omni::Action::DeployGlobalContract(deploy) => {
            near_primitives::transaction::Action::DeployGlobalContract(
                near_primitives::action::DeployGlobalContractAction {
                    code: std::sync::Arc::from(deploy.code.into_boxed_slice()),
                    deploy_mode: omni_global_contract_deploy_mode_to_near(deploy.deploy_mode),
                },
            )
        }
        omni::Action::UseGlobalContract(use_global) => {
            near_primitives::transaction::Action::UseGlobalContract(Box::new(
                near_primitives::action::UseGlobalContractAction {
                    contract_identifier: omni_global_contract_identifier_to_near(
                        use_global.contract_identifier,
                    ),
                },
            ))
        }
    }
}

fn near_public_key_to_omni(key: near_crypto::PublicKey) -> near_crypto::PublicKey {
    key
}

fn omni_public_key_to_near(key: near_crypto::PublicKey) -> near_crypto::PublicKey {
    key
}

fn near_access_key_to_omni(key: near_primitives::account::AccessKey) -> omni::AccessKey {
    omni::AccessKey {
        nonce: omni::U64(key.nonce),
        permission: match key.permission {
            near_primitives::account::AccessKeyPermission::FullAccess => {
                omni::AccessKeyPermission::FullAccess
            }
            near_primitives::account::AccessKeyPermission::FunctionCall(fc) => {
                omni::AccessKeyPermission::FunctionCall(omni::FunctionCallPermission {
                    allowance: fc.allowance.map(|a| omni::U128(a.as_yoctonear())),
                    receiver_id: fc.receiver_id,
                    method_names: fc.method_names,
                })
            }
        },
    }
}

fn omni_access_key_to_near(key: omni::AccessKey) -> near_primitives::account::AccessKey {
    near_primitives::account::AccessKey {
        nonce: key.nonce.0,
        permission: match key.permission {
            omni::AccessKeyPermission::FullAccess => {
                near_primitives::account::AccessKeyPermission::FullAccess
            }
            omni::AccessKeyPermission::FunctionCall(fc) => {
                near_primitives::account::AccessKeyPermission::FunctionCall(
                    near_primitives::account::FunctionCallPermission {
                        allowance: fc.allowance.map(|a| near_token::NearToken::from_yoctonear(a.0)),
                        receiver_id: fc.receiver_id,
                        method_names: fc.method_names,
                    },
                )
            }
        },
    }
}

fn near_signed_delegate_action_to_omni(
    action: near_primitives::action::delegate::SignedDelegateAction,
) -> omni::SignedDelegateAction {
    omni::SignedDelegateAction {
        delegate_action: omni::DelegateAction {
            sender_id: action.delegate_action.sender_id,
            receiver_id: action.delegate_action.receiver_id,
            actions: action
                .delegate_action
                .actions
                .into_iter()
                .map(|a| {
                    near_action_to_omni(a.into())
                        .expect("Failed to convert action")
                        .try_into()
                        .expect("Delegate action should not contain another delegate action")
                })
                .collect(),
            nonce: omni::U64(action.delegate_action.nonce),
            max_block_height: omni::U64(action.delegate_action.max_block_height),
            public_key: near_public_key_to_omni(action.delegate_action.public_key),
        },
        signature: near_signature_to_omni(action.signature),
    }
}

fn omni_signed_delegate_action_to_near(
    action: omni::SignedDelegateAction,
) -> near_primitives::action::delegate::SignedDelegateAction {
    near_primitives::action::delegate::SignedDelegateAction {
        delegate_action: near_primitives::action::delegate::DelegateAction {
            sender_id: action.delegate_action.sender_id,
            receiver_id: action.delegate_action.receiver_id,
            actions: action
                .delegate_action
                .actions
                .into_iter()
                .map(|a| {
                    omni_action_to_near(a.0)
                        .try_into()
                        .expect("Delegate action should not contain another delegate action")
                })
                .collect(),
            nonce: action.delegate_action.nonce.0,
            max_block_height: action.delegate_action.max_block_height.0,
            public_key: omni_public_key_to_near(action.delegate_action.public_key),
        },
        signature: omni_signature_to_near(action.signature),
    }
}

fn near_signature_to_omni(sig: near_crypto::Signature) -> near_crypto::Signature {
    sig
}

fn omni_signature_to_near(sig: near_crypto::Signature) -> near_crypto::Signature {
    sig
}

fn near_global_contract_deploy_mode_to_omni(
    mode: near_primitives::action::GlobalContractDeployMode,
) -> omni::GlobalContractDeployMode {
    match mode {
        near_primitives::action::GlobalContractDeployMode::CodeHash => {
            omni::GlobalContractDeployMode::CodeHash
        }
        near_primitives::action::GlobalContractDeployMode::AccountId => {
            omni::GlobalContractDeployMode::AccountId
        }
    }
}

fn omni_global_contract_deploy_mode_to_near(
    mode: omni::GlobalContractDeployMode,
) -> near_primitives::action::GlobalContractDeployMode {
    match mode {
        omni::GlobalContractDeployMode::CodeHash => {
            near_primitives::action::GlobalContractDeployMode::CodeHash
        }
        omni::GlobalContractDeployMode::AccountId => {
            near_primitives::action::GlobalContractDeployMode::AccountId
        }
    }
}

fn near_global_contract_identifier_to_omni(
    id: near_primitives::action::GlobalContractIdentifier,
) -> omni::GlobalContractIdentifier {
    match id {
        near_primitives::action::GlobalContractIdentifier::CodeHash(hash) => {
            omni::GlobalContractIdentifier::CodeHash(omni::BlockHash(hash.0))
        }
        near_primitives::action::GlobalContractIdentifier::AccountId(account_id) => {
            omni::GlobalContractIdentifier::AccountId(account_id)
        }
    }
}

fn omni_global_contract_identifier_to_near(
    id: omni::GlobalContractIdentifier,
) -> near_primitives::action::GlobalContractIdentifier {
    match id {
        omni::GlobalContractIdentifier::CodeHash(hash) => {
            near_primitives::action::GlobalContractIdentifier::CodeHash(
                near_primitives::hash::CryptoHash(hash.0),
            )
        }
        omni::GlobalContractIdentifier::AccountId(account_id) => {
            near_primitives::action::GlobalContractIdentifier::AccountId(account_id)
        }
    }
}

/// Convert near_primitives::transaction::TransactionV0 to our internal Transaction
pub fn near_transaction_to_omni(
    tx: near_primitives::transaction::TransactionV0,
) -> omni::Transaction {
    omni::Transaction {
        signer_id: tx.signer_id,
        signer_public_key: near_public_key_to_omni(tx.public_key),
        nonce: omni::U64(tx.nonce),
        receiver_id: tx.receiver_id,
        block_hash: omni::BlockHash(tx.block_hash.0),
        actions: tx.actions.into_iter().map(|a| near_action_to_omni(a).expect("Failed to convert action")).collect(),
    }
}

/// Convert our internal Transaction to near_primitives::transaction::TransactionV0
pub fn omni_transaction_to_near(
    tx: omni::Transaction,
) -> near_primitives::transaction::TransactionV0 {
    near_primitives::transaction::TransactionV0 {
        signer_id: tx.signer_id,
        public_key: omni_public_key_to_near(tx.signer_public_key),
        nonce: tx.nonce.0,
        receiver_id: tx.receiver_id,
        block_hash: near_primitives::hash::CryptoHash(tx.block_hash.0),
        actions: tx.actions.into_iter().map(omni_action_to_near).collect(),
    }
}

/// Error type for transaction conversion
#[derive(Debug, Clone)]
pub enum TransactionConversionError {
    /// Transaction V1 is not yet supported
    UnsupportedTransactionV1,
}

impl std::fmt::Display for TransactionConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionConversionError::UnsupportedTransactionV1 => {
                write!(f, "Transaction V1 is not yet supported in internal transaction types")
            }
        }
    }
}

impl std::error::Error for TransactionConversionError {}

/// Convert near_primitives::transaction::SignedTransaction to our internal SignedTransaction
/// Returns an error if the transaction version is not supported
pub fn near_signed_transaction_to_omni(
    signed_tx: near_primitives::transaction::SignedTransaction,
) -> Result<omni::SignedTransaction, TransactionConversionError> {
    let transaction = match signed_tx.transaction {
        near_primitives::transaction::Transaction::V0(v0) => near_transaction_to_omni(v0),
        near_primitives::transaction::Transaction::V1(_) => {
            return Err(TransactionConversionError::UnsupportedTransactionV1);
        }
    };
    Ok(omni::SignedTransaction {
        transaction,
        signature: near_signature_to_omni(signed_tx.signature),
    })
}

/// Convert omni_transaction SignedTransaction to near_primitives::transaction::SignedTransaction
pub fn omni_signed_transaction_to_near(
    signed_tx: omni::SignedTransaction,
) -> near_primitives::transaction::SignedTransaction {
    let transaction_v0 = omni_transaction_to_near(signed_tx.transaction);
    near_primitives::transaction::SignedTransaction::new(
        omni_signature_to_near(signed_tx.signature),
        near_primitives::transaction::Transaction::V0(transaction_v0),
    )
}
