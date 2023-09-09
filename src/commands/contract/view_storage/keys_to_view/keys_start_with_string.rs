#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::ViewStorageContext)]
#[interactive_clap(output_context = KeysStartWithStringContext)]
pub struct KeysStartWithString {
    /// Enter the string that the keys begin with (for example, "S"):
    keys_begin_with: String,
    #[interactive_clap(subcommand)]
    output_format: super::super::output_format::OutputFormat,
}

#[derive(Debug, Clone)]
pub struct KeysStartWithStringContext(super::KeysContext);

impl KeysStartWithStringContext {
    pub fn from_previous_context(
        previous_context: super::super::ViewStorageContext,
        scope: &<KeysStartWithString as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(super::KeysContext {
            global_context: previous_context.global_context,
            contract_account_id: previous_context.contract_account_id,
            prefix: near_primitives::types::StoreKey::from(
                scope.keys_begin_with.clone().into_bytes(),
            ),
        }))
    }
}

impl From<KeysStartWithStringContext> for super::KeysContext {
    fn from(item: KeysStartWithStringContext) -> Self {
        item.0
    }
}
