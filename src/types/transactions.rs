/// Internal transaction and action types compatible with omni-transaction structure
/// These types match the omni-transaction-rs types but without the schemars dependency
use borsh::{BorshDeserialize, BorshSerialize};
use near_crypto::{PublicKey, Signature};
use serde::{Deserialize, Serialize};

/// U64 wrapper for compatibility with omni-transaction
#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Eq,
)]
pub struct U64(pub u64);

/// U128 wrapper for compatibility with omni-transaction
#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Eq,
)]
pub struct U128(pub u128);

/// BlockHash wrapper
#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Eq,
)]
pub struct BlockHash(pub [u8; 32]);

/// NEAR transaction structure matching omni-transaction-rs
#[derive(Serialize, Deserialize, Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Transaction {
    pub signer_id: near_primitives::types::AccountId,
    #[serde(rename = "public_key")]
    pub signer_public_key: PublicKey,
    pub nonce: U64,
    pub receiver_id: near_primitives::types::AccountId,
    pub block_hash: BlockHash,
    pub actions: Vec<Action>,
}

/// Signed NEAR transaction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SignedTransaction {
    pub transaction: Transaction,
    pub signature: Signature,
}

/// Action enum matching omni-transaction-rs structure
#[derive(Serialize, Deserialize, Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub enum Action {
    CreateAccount(CreateAccountAction),
    DeployContract(DeployContractAction),
    FunctionCall(Box<FunctionCallAction>),
    Transfer(TransferAction),
    Stake(Box<StakeAction>),
    AddKey(Box<AddKeyAction>),
    DeleteKey(Box<DeleteKeyAction>),
    DeleteAccount(DeleteAccountAction),
    Delegate(Box<SignedDelegateAction>),
    DeployGlobalContract(DeployGlobalContractAction),
    UseGlobalContract(Box<UseGlobalContractAction>),
}

#[derive(Serialize, Deserialize, Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct CreateAccountAction {}

#[derive(Serialize, Deserialize, Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct DeployContractAction {
    pub code: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct FunctionCallAction {
    pub method_name: String,
    pub args: Vec<u8>,
    pub gas: U64,
    pub deposit: U128,
}

#[derive(Serialize, Deserialize, Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct TransferAction {
    pub deposit: U128,
}

#[derive(Serialize, Deserialize, Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct StakeAction {
    pub stake: U128,
    pub public_key: PublicKey,
}

#[derive(Serialize, Deserialize, Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct AddKeyAction {
    pub public_key: PublicKey,
    pub access_key: AccessKey,
}

#[derive(Serialize, Deserialize, Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct AccessKey {
    pub nonce: U64,
    pub permission: AccessKeyPermission,
}

#[derive(Serialize, Deserialize, Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub enum AccessKeyPermission {
    FunctionCall(FunctionCallPermission),
    FullAccess,
}

#[derive(Serialize, Deserialize, Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct FunctionCallPermission {
    pub allowance: Option<U128>,
    pub receiver_id: String,
    pub method_names: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct DeleteKeyAction {
    pub public_key: PublicKey,
}

#[derive(Serialize, Deserialize, Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct DeleteAccountAction {
    pub beneficiary_id: near_primitives::types::AccountId,
}

#[derive(Serialize, Deserialize, Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct DeployGlobalContractAction {
    pub code: Vec<u8>,
    pub deploy_mode: GlobalContractDeployMode,
}

#[derive(Serialize, Deserialize, Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct UseGlobalContractAction {
    pub contract_identifier: GlobalContractIdentifier,
}

#[derive(Serialize, Deserialize, Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub enum GlobalContractDeployMode {
    CodeHash,
    AccountId,
}

#[derive(Serialize, Deserialize, Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub enum GlobalContractIdentifier {
    CodeHash(BlockHash),
    AccountId(near_primitives::types::AccountId),
}

#[derive(Serialize, Deserialize, Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct NonDelegateAction(pub Action);

impl TryFrom<Action> for NonDelegateAction {
    type Error = ();
    fn try_from(action: Action) -> Result<Self, Self::Error> {
        if let Action::Delegate(_) = action {
            return Err(());
        }
        Ok(Self(action))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct DelegateAction {
    pub sender_id: near_primitives::types::AccountId,
    pub receiver_id: near_primitives::types::AccountId,
    pub actions: Vec<NonDelegateAction>,
    pub nonce: U64,
    pub max_block_height: U64,
    pub public_key: PublicKey,
}

#[derive(Serialize, Deserialize, Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct SignedDelegateAction {
    pub delegate_action: DelegateAction,
    pub signature: Signature,
}

impl Transaction {
    pub fn build_for_signing(&self) -> Vec<u8> {
        borsh::to_vec(self).expect("failed to serialize NEAR transaction")
    }
}
