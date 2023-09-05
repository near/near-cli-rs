use color_eyre::eyre::Context;

use near_primitives::serialize::base64_display;
use prettytable::Table;


use crate::common::JsonRpcClientExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ViewStateContext)]
pub struct ViewState {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the contract account ID?
    contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct ViewStateContext(crate::network_view_at_block::ArgsForViewContext);

impl ViewStateContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<ViewState as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let contract_account_id: near_primitives::types::AccountId = scope.contract_account_id.clone().into();

            move |network_config, block_reference| {
                let query_view_method_response = network_config
                    .json_rpc_client()
                    .blocking_call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                        block_reference: block_reference.clone(),
                        request: near_primitives::views::QueryRequest::ViewState {
                            account_id: contract_account_id.clone(),
                            prefix: near_primitives::types::StoreKey::from(Vec::new()),
                            include_proof: false,
                        },
                    })
                    .wrap_err_with(|| format!("Failed to fetch query ViewState for <{contract_account_id}>"))?;
                    if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewState(result) =
                        query_view_method_response.kind
                    {
                        eprintln!("Contract state (values):");
                        eprintln!(
                            "{}",
                            // result.values[0]
                            // serde_json::to_string_pretty(&result.values[0].key)?
                            serde_json::to_string_pretty(&result.values)?
                            // base64_display(&result.values[0].key.as_slice())
                        );



                        for value in &result.values {
                            eprintln!("{}: {}", String::from_utf8_lossy(&value.key), String::from_utf8_lossy(&value.value));
                            // // let qwe = serde_json::from_slice::<near_primitives::types::StoreKey>(&result.values[0].key)?;
                            // eprintln!(
                            //     "{}: {}",
                            //     // result.values[0]
                            //     // serde_json::to_string_pretty(&result.values[0].key)?
                            //     // serde_json::to_string_pretty(value)?
                            //     base64_display(&value.key.as_slice()),
                            //     base64_display(&value.value.as_slice())
                            // );
                        }


                        let mut table = Table::new();
                        table.set_titles(prettytable::row![Fg=>"key", "value"]);
                        for value in &result.values {
                            table.add_row(prettytable::row![
                                Fg->String::from_utf8_lossy(&value.key),
                                String::from_utf8_lossy(&value.value)
                            ]);
                    
                            // eprintln!("{}: {}", String::from_utf8_lossy(&value.key), String::from_utf8_lossy(&value.value));
                        }
                        table.set_format(*prettytable::format::consts::FORMAT_NO_COLSEP);
                        table.printstd();
                    




                        eprintln!(
                            "\nContract state (proof):\n{:#?}\n",
                            &result.proof
                        );
                        println!("{}", "'".to_string());
                    } else {
                        return Err(color_eyre::Report::msg("Error call result".to_string()));
                    };
                Ok(())
            }
        });

        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.config,
            interacting_with_account_ids: vec![scope.contract_account_id.clone().into()],
            on_after_getting_block_reference_callback,
        }))
    }
}

impl From<ViewStateContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: ViewStateContext) -> Self {
        item.0
    }
}

impl ViewState {
    pub fn input_contract_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is the contract account ID?",
        )
    }
}
