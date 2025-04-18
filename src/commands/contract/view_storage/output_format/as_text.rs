use color_eyre::{eyre::Context, owo_colors::OwoColorize};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::keys_to_view::KeysContext)]
#[interactive_clap(output_context = AsTextContext)]
pub struct AsText {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct AsTextContext(crate::network_view_at_block::ArgsForViewContext);

impl AsTextContext {
    pub fn from_previous_context(
        previous_context: super::super::keys_to_view::KeysContext,
        _scope: &<AsText as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let contract_account_id = previous_context.contract_account_id.clone();
            let prefix = previous_context.prefix;

            move |network_config, block_reference| {
                let query_view_method_response =
                    super::get_contract_state(&contract_account_id, prefix.clone(), network_config, block_reference.clone())?;

                if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewState(result) =
                    query_view_method_response.kind
                {
                    let mut info_str = String::new();
                    for value in &result.values {
                        info_str.push_str(&format!("\n\tkey:   {}", key_value_to_string(&value.key)?.green()));
                        info_str.push_str(&format!("\n\tvalue: {}", key_value_to_string(&value.value)?.yellow()));
                        info_str.push_str("\n\t--------------------------------");
                    }
                    tracing::info!(
                        parent: &tracing::Span::none(),
                        "Contract state (values):{}\n",
                        crate::common::indent_payload(&info_str)
                    );
                    tracing::info!(
                        parent: &tracing::Span::none(),
                        "Contract state (proof):\n{}\n",
                        crate::common::indent_payload(&format!("{:#?}", result.proof))
                    );
                } else {
                    return Err(color_eyre::Report::msg("Error call result".to_string()));
                };
                Ok(())
            }
        });

        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.global_context.config,
            interacting_with_account_ids: vec![previous_context.contract_account_id],
            on_after_getting_block_reference_callback,
        }))
    }
}

impl From<AsTextContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: AsTextContext) -> Self {
        item.0
    }
}

fn key_value_to_string(slice: &[u8]) -> color_eyre::eyre::Result<String> {
    String::from_utf8(
        slice
            .iter()
            .flat_map(|b| std::ascii::escape_default(*b))
            .collect::<Vec<u8>>(),
    )
    .wrap_err("Wrong format. utf-8 is expected.")
}
