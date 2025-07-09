use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod base64;
mod utf8;

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::SignNep413Context)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select the message encoding type:
pub enum MessageType {
    #[strum_discriminants(strum(message = "utf8     - The message is a plain UTF-8 string"))]
    /// The message is a plain UTF-8 string
    Utf8(self::utf8::Utf8),
    #[strum_discriminants(strum(message = "base64   - The message is a base64-encoded string"))]
    /// The message is a base64-encoded string
    Base64(self::base64::Base64),
}

#[derive(Debug, Clone)]
pub struct MessageTypeContext {
    pub global_context: crate::GlobalContext,
    pub payload: super::NEP413Payload,
    pub signer_id: near_primitives::types::AccountId,
}

impl MessageTypeContext {
    pub fn from_previous_context(
        previous_context: super::SignNep413Context,
        _scope: &<MessageType as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            payload: previous_context.payload,
            signer_id: previous_context.signer_id,
        })
    }
}

impl From<super::SignNep413Context> for MessageTypeContext {
    fn from(item: super::SignNep413Context) -> Self {
        Self {
            global_context: item.global_context,
            payload: item.payload,
            signer_id: item.signer_id,
        }
    }
}
