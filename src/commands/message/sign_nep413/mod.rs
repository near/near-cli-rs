use near_crypto::{SecretKey, Signature};
use near_primitives::borsh::{self, BorshDeserialize, BorshSerialize};
use near_primitives::hash::hash;
use serde::Serialize;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod signature_options;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = SignNep413Context)]
pub struct SignNep413 {
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_default_input_arg)]
    /// Message encoding:
    encoding: MessageEncoding,
    /// The message to sign:
    message: String,
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
    sign_with: self::signature_options::SignWith,
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
        let message = match scope.encoding {
            MessageEncoding::Utf8 => scope.message.clone(),
            MessageEncoding::Base64 => String::from_utf8(
                near_primitives::serialize::from_base64(&scope.message).map_err(|e| {
                    color_eyre::eyre::eyre!("Failed to decode base64 message: {}", e)
                })?,
            )
            .map_err(|e| color_eyre::eyre::eyre!("Message is not valid UTF-8: {}", e))?,
        };

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
            message,
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

#[derive(Debug, Clone, EnumDiscriminants, clap::ValueEnum)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum MessageEncoding {
    Utf8,
    Base64,
}

impl interactive_clap::ToCli for MessageEncoding {
    type CliVariant = Self;
}

impl std::fmt::Display for MessageEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Utf8 => write!(f, "utf8"),
            Self::Base64 => write!(f, "base64"),
        }
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

impl SignNep413 {
    fn input_encoding(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<MessageEncoding>> {
        Ok(Some(
            inquire::Select::new(
                "How is the message encoded?",
                vec![MessageEncoding::Utf8, MessageEncoding::Base64],
            )
            .prompt()?,
        ))
    }
}
