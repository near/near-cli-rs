use color_eyre::{eyre::Context, owo_colors::OwoColorize};

use crate::common::JsonRpcClientExt;

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
                let query_view_method_response = network_config
                    .json_rpc_client()
                    .blocking_call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                        block_reference: block_reference.clone(),
                        request: near_primitives::views::QueryRequest::ViewState {
                            account_id: contract_account_id.clone(),
                            prefix: prefix.clone(),
                            include_proof: false,
                        },
                    })
                    .wrap_err_with(|| format!("Failed to fetch query ViewState for <{contract_account_id}>"))?;
                    if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewState(result) =
                        query_view_method_response.kind
                    {
                        eprintln!("Contract state (values):");
                        for value in &result.values {
                            let key = String::from_utf8(
                                value.key
                                .as_slice()
                                .iter()
                                .flat_map(|b| std::ascii::escape_default(*b))
                                .collect::<Vec<u8>>(),
                            )
                            .wrap_err("Wrong format. utf-8 is expected.")?;
                            eprintln!("key:\n{}", key.green());

                            let val = String::from_utf8(
                                value.value
                                .as_slice()
                                .iter()
                                .flat_map(|b| std::ascii::escape_default(*b))
                                .collect::<Vec<u8>>(),
                            )
                            .wrap_err("Wrong format. utf-8 is expected.")?;
                            eprintln!("value:\n{}", val.yellow());

                            eprintln!("--------------------------------");
                        }
                        eprintln!(
                            "\nContract state (proof):\n{:#?}\n",
                            &result.proof
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
