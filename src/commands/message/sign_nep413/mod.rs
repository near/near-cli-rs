use near_crypto::{SecretKey, Signature};
use near_primitives::borsh::{self, BorshDeserialize, BorshSerialize};
use near_primitives::hash::hash;
use serde::Serialize;

pub mod message_type;
pub mod signature_options;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = SignNep413Context)]
pub struct SignNep413 {
    #[interactive_clap(long)]
    /// A 32-byte nonce as a base64-encoded string:
    nonce: crate::types::base64_bytes::Base64Bytes,
    #[interactive_clap(long)]
    /// The recipient of the message (e.g. "alice.near" or "myapp.com"):
    recipient: String,
    #[interactive_clap(long)]
    /// Which account to sign the message with:
    signer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    message_type: self::message_type::MessageType,
}

#[derive(Debug, Clone)]
pub struct SignNep413Context {
    pub global_context: crate::GlobalContext,
    pub payload: NEP413Payload,
    pub signer_id: near_primitives::types::AccountId,
}

impl SignNep413Context {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<SignNep413 as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let nonce_bytes = scope.nonce.as_bytes();
        if nonce_bytes.len() != 32 {
            return Err(color_eyre::eyre::eyre!(
                "Invalid nonce length: expected 32 bytes, got {}",
                nonce_bytes.len()
            ));
        }
        let mut nonce = [0u8; 32];
        nonce.copy_from_slice(nonce_bytes);

        let payload = NEP413Payload {
            message: String::new(),
            nonce,
            recipient: scope.recipient.clone(),
            callback_url: None,
        };

        Ok(Self {
            global_context: previous_context,
            payload,
            signer_id: scope.signer_account_id.clone().into(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct SignedMessage {
    #[serde(rename = "accountId")]
    pub account_id: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub signature: String,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct NEP413Payload {
    pub message: String,
    pub nonce: [u8; 32],
    pub recipient: String,
    pub callback_url: Option<String>,
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
