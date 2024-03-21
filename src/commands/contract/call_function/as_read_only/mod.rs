use color_eyre::eyre::Context;
use inquire::Select;
use interactive_clap::ToCliArgs;

use crate::common::CallResultExt;
use crate::common::JsonRpcClientExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = CallFunctionViewContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct CallFunctionView {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the contract account ID?
    contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(skip_default_input_arg)]
    #[interactive_clap(flatten)]
    /// Select function
    function: Function,
}

#[derive(Clone)]
pub struct CallFunctionViewContext {
    global_context: crate::GlobalContext,
    contract_account_id: near_primitives::types::AccountId,
}

impl CallFunctionViewContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<CallFunctionView as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            contract_account_id: scope.contract_account_id.clone().into(),
        })
    }
}

impl interactive_clap::FromCli for CallFunctionView {
    type FromCliContext = crate::GlobalContext;
    type FromCliError = color_eyre::eyre::Error;
    fn from_cli(
        optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.clone().unwrap_or_default();

        if clap_variant.contract_account_id.is_none() {
            clap_variant.contract_account_id = match Self::input_contract_account_id(&context) {
                Ok(Some(contract_account_id)) => Some(contract_account_id),
                Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let contract_account_id = clap_variant
            .contract_account_id
            .clone()
            .expect("Unexpected error");

        let output_context = CallFunctionViewContext {
            global_context: context,
            contract_account_id: contract_account_id.clone().into(),
        };

        let function = match Function::from_cli(
            optional_clap_variant.unwrap_or_default().function,
            output_context,
        ) {
            interactive_clap::ResultFromCli::Ok(function) => function,
            interactive_clap::ResultFromCli::Cancel(optional_function) => {
                clap_variant.function = optional_function;
                return interactive_clap::ResultFromCli::Cancel(Some(clap_variant));
            }
            interactive_clap::ResultFromCli::Back => return interactive_clap::ResultFromCli::Back,
            interactive_clap::ResultFromCli::Err(optional_function, err) => {
                clap_variant.function = optional_function;
                return interactive_clap::ResultFromCli::Err(Some(clap_variant), err);
            }
        };
        interactive_clap::ResultFromCli::Ok(CliCallFunctionView {
            contract_account_id: Some(contract_account_id),
            function: Some(function),
        })
    }
}

impl CallFunctionView {
    pub fn input_contract_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is the contract account ID?",
        )
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = CallFunctionViewContext)]
#[interactive_clap(output_context = FunctionContext)]
pub struct Function {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the name of the function?
    function_name: String,
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_default_input_arg)]
    /// How do you want to pass the function call arguments?
    function_args_type: super::call_function_args_type::FunctionArgsType,
    /// Enter the arguments to this function:
    function_args: String,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

impl std::fmt::Display for CliFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let cli_args: Vec<String> = self.to_cli_args().into();
        write!(f, "{}", cli_args.join(" "))
    }
}

#[derive(Clone)]
pub struct FunctionContext(crate::network_view_at_block::ArgsForViewContext);

impl FunctionContext {
    pub fn from_previous_context(
        previous_context: CallFunctionViewContext,
        scope: &<Function as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let function_args = scope.function_args.clone();
            let function_args_type = scope.function_args_type.clone();
            let account_id: near_primitives::types::AccountId = previous_context.contract_account_id.clone();
            let function_name = scope.function_name.clone();

            move |network_config, block_reference| {
                let args = super::call_function_args_type::function_args(
                    function_args.clone(),
                    function_args_type.clone(),
                )?;

                let call_result = network_config
                .json_rpc_client()
                .blocking_call_view_function(
                    &account_id,
                    &function_name,
                    args,
                    block_reference.clone(),
                )
                .wrap_err_with(|| {
                    format!("Failed to fetch query for view method: '{}' (contract <{}> on network <{}>)",
                        function_name,
                        account_id,
                        network_config.network_name
                    )
                })?;
                call_result.print_logs();
                eprintln!("Result:");
                if call_result.result.is_empty() {
                    eprintln!("Empty result");
                } else if let Ok(json_result) = call_result.parse_result_from_json::<serde_json::Value>() {
                    println!("{}", serde_json::to_string_pretty(&json_result)?);
                } else if let Ok(string_result) = String::from_utf8(call_result.result) {
                    println!("{string_result}");
                } else {
                    eprintln!("The returned value is not printable (binary data)");
                }
                eprintln!("--------------");
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

impl From<FunctionContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: FunctionContext) -> Self {
        item.0
    }
}

impl Function {
    fn input_function_args_type(
        _context: &CallFunctionViewContext,
    ) -> color_eyre::eyre::Result<Option<super::call_function_args_type::FunctionArgsType>> {
        super::call_function_args_type::input_function_args_type()
    }

    fn input_function_name(
        _context: &CallFunctionViewContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        let options = vec!["qwe", "asd", "zxc", "new", "unsafe_self_upgrade", "get_num"];

        let function_name =
            Select::new("Select the viewing function for your contract:", options).prompt()?;

        Ok(Some(function_name.to_string()))
    }
}
