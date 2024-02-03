use color_eyre::eyre::Context;

use crate::common::CallResultExt;
use crate::common::JsonRpcClientExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ContractContext)]
pub struct Contract {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the contract account ID?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

impl Contract {
    pub fn input_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is the contract account ID?",
        )
    }
}

#[derive(Clone)]
pub struct ContractContext(crate::network_view_at_block::ArgsForViewContext);

impl ContractContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<Contract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let account_id = scope.account_id.clone();

            move |network_config, block_reference| {
                let query_view_code_response = network_config
                    .json_rpc_client()
                    .blocking_call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                        block_reference: block_reference.clone(),
                        request: near_primitives::views::QueryRequest::ViewCode {
                            account_id: account_id.clone().into(),
                        },
                    })
                    .wrap_err_with(|| format!("Failed to fetch query ViewCode for <{}> on network <{}>", &account_id, network_config.network_name))?;

                let mut table = prettytable::Table::new();
                table.set_format(*prettytable::format::consts::FORMAT_NO_COLSEP);

                table.add_row(prettytable::row![
                    Fy->account_id,
                    format!("At block #{}\n({})", query_view_code_response.block_height, query_view_code_response.block_hash)
                ]);


                if let Ok(contract_source_metadata_call_result) = network_config
                    .json_rpc_client()
                    .blocking_call_view_function(
                        &account_id.clone().into(),
                        "contract_source_metadata",
                        vec![],
                        block_reference.clone(),
                    ) {
                        println!("contract_source_metadata:");
                        if let Ok(contract_source_metadata) = contract_source_metadata_call_result.parse_result_from_json::<near_contract_standards::contract_metadata::ContractSourceMetadata>() {
                            println!("{}", serde_json::to_string_pretty(&contract_source_metadata)?);
                        } else {
                            eprintln!("The returned value is not printable (binary data)");
                        }

                        if let Ok(call_result) = network_config
                            .json_rpc_client()
                            .blocking_call_view_function(
                                &account_id.clone().into(),
                                "__contract_abi",
                                vec![],
                                block_reference.clone(),
                            ) {
                                if let Ok(abi_root) = serde_json::from_slice::<near_abi::AbiRoot>(&zstd::decode_all(&call_result.result[..])?) {
                                    println!("{:#?}", abi_root);
                                }
                            }
                            table.printstd();
                            return Ok(());
                    }


                let contract_code_view =
                    if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewCode(result) =
                        query_view_code_response.kind
                    {
                        result
                    } else {
                        return Err(color_eyre::Report::msg("Error call result".to_string()));
                    };


                table.set_titles(prettytable::row![Fg=>"metod name", "Arguments"]);

                for function in wasmer::Module::from_binary(&wasmer::Store::default(), &contract_code_view.code)
                    .wrap_err_with(|| format!("Could not create new WebAssembly module from Wasm binary for contract <{account_id}>."))?
                    .exports()
                    // .filter(|e| matches!(e.ty(), wasmer::ExternType::Function(_fty)))
                {
                    table.add_row(prettytable::row![
                        Fy->function.name(),
                        ""
                    ]);
                }


                table.printstd();
                Ok(())
            }

        });
        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.config,
            on_after_getting_block_reference_callback,
            interacting_with_account_ids: vec![scope.account_id.clone().into()],
        }))
    }
}

impl From<ContractContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: ContractContext) -> Self {
        item.0
    }
}
