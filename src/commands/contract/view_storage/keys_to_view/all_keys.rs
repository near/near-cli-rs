#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::ViewStorageContext)]
#[interactive_clap(output_context = AllKeysContext)]
pub struct AllKeys {
    #[interactive_clap(subcommand)]
    output_format: super::super::output_format::OutputFormat,
}

#[derive(Debug, Clone)]
pub struct AllKeysContext(super::KeysContext);

impl AllKeysContext {
    pub fn from_previous_context(
        previous_context: super::super::ViewStorageContext,
        _scope: &<AllKeys as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(super::KeysContext {
            global_context: previous_context.global_context,
            contract_account_id: previous_context.contract_account_id,
            prefix: near_primitives::types::StoreKey::from(Vec::new()),
        }))
    }
}

impl From<AllKeysContext> for super::KeysContext {
    fn from(item: AllKeysContext) -> Self {
        item.0
    }
}
