use near_crypto::{SecretKey, Signature};
use near_primitives::borsh::{self, BorshDeserialize, BorshSerialize};
use near_primitives::hash::hash;
use serde::Serialize;

pub mod message_type;
pub mod nonce;
pub mod recipient;
pub mod signature_options;
pub mod signer;

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct NEP413Payload {
    pub message: String,
    pub nonce: [u8; 32],
    pub recipient: String,
    pub callback_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SignedMessage {
    #[serde(rename = "accountId")]
    pub account_id: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub signature: String,
}

pub fn sign_nep413_payload(
    payload: &NEP413Payload,
    secret_key: &SecretKey,
) -> color_eyre::eyre::Result<Signature> {
    const NEP413_SIGN_MESSAGE_PREFIX: u32 = (1u32 << 31u32) + 413u32;
    let mut bytes = NEP413_SIGN_MESSAGE_PREFIX.to_le_bytes().to_vec();
    borsh::to_writer(&mut bytes, payload)?;
    let hash = hash(&bytes);
    let signature = secret_key.sign(hash.as_ref());
    Ok(signature)
}

#[cfg(feature = "ledger")]
impl From<NEP413Payload> for near_ledger::NEP413Payload {
    fn from(payload: NEP413Payload) -> Self {
        Self {
            message: payload.message,
            nonce: payload.nonce,
            recipient: payload.recipient,
            callback_url: payload.callback_url,
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = SignNep413Context)]
pub struct SignNep413 {
    #[interactive_clap(subcommand)]
    message_type: self::message_type::MessageType,
}

#[derive(Debug, Clone)]
pub struct SignNep413Context {
    global_context: crate::GlobalContext,
}

impl SignNep413Context {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        _scope: &<SignNep413 as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FinalSignNep413Context {
    pub global_context: crate::GlobalContext,
    pub payload: NEP413Payload,
    pub signer_id: near_primitives::types::AccountId,
}
