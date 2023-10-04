#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::ViewStorageContext)]
#[interactive_clap(output_context = KeysStartWithBytesAsBase64Context)]
pub struct KeysStartWithBytesAsBase64 {
    /// Enter the string that the keys begin with Base64 bytes (for example, "Uw=="):
    keys_begin_with: crate::types::base64_bytes::Base64Bytes,
    #[interactive_clap(subcommand)]
    output_format: super::super::output_format::OutputFormat,
}

#[derive(Debug, Clone)]
pub struct KeysStartWithBytesAsBase64Context(super::KeysContext);

impl KeysStartWithBytesAsBase64Context {
    pub fn from_previous_context(
        previous_context: super::super::ViewStorageContext,
        scope: &<KeysStartWithBytesAsBase64 as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(super::KeysContext {
            global_context: previous_context.global_context,
            contract_account_id: previous_context.contract_account_id,
            prefix: near_primitives::types::StoreKey::from(scope.keys_begin_with.into_bytes()),
        }))
    }
}

impl From<KeysStartWithBytesAsBase64Context> for super::KeysContext {
    fn from(item: KeysStartWithBytesAsBase64Context) -> Self {
        item.0
    }
}
