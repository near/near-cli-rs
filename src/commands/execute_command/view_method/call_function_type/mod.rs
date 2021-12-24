use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::operation_mode::online_mode::select_server::ExecuteViewMethodCommandNetworkContext)]
pub struct CallFunctionView {
    method_name: String,
    function_args: String,
    #[interactive_clap(subcommand)]
    selected_block_id: super::block_id::BlockId,
}

impl CallFunctionView {
    fn input_method_name(
        _context: &super::operation_mode::online_mode::select_server::ExecuteViewMethodCommandNetworkContext,
    ) -> color_eyre::eyre::Result<String> {
        println!();
        Ok(Input::new()
            .with_prompt("Enter a method name")
            .interact_text()?)
    }

    fn input_function_args(
        _context: &super::operation_mode::online_mode::select_server::ExecuteViewMethodCommandNetworkContext,
    ) -> color_eyre::eyre::Result<String> {
        println!();
        Ok(Input::new()
            .with_prompt("Enter args for function")
            .interact_text()?)
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
        contract_account_id: near_primitives::types::AccountId,
    ) -> crate::CliResult {
        self.selected_block_id
            .process(
                contract_account_id,
                network_connection_config,
                self.method_name,
                self.function_args.into_bytes(),
            )
            .await
    }
}
