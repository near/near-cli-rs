use dialoguer::Input;

/// Call view function
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliCallFunctionView {
    method_name: Option<String>,
    function_args: Option<String>,
    #[clap(subcommand)]
    selected_block_id: Option<super::block_id::CliBlockId>,
}

#[derive(Debug, Clone)]
pub struct CallFunctionView {
    method_name: String,
    function_args: Vec<u8>,
    selected_block_id: super::block_id::BlockId,
}

impl CliCallFunctionView {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .selected_block_id
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(function_args) = &self.function_args {
            args.push_front(function_args.to_owned());
        };
        if let Some(method_name) = &self.method_name {
            args.push_front(method_name.to_string());
        };
        args
    }
}

impl From<CallFunctionView> for CliCallFunctionView {
    fn from(call_function_view: CallFunctionView) -> Self {
        Self {
            method_name: Some(call_function_view.method_name),
            function_args: Some(
                String::from_utf8(call_function_view.function_args).unwrap_or_default(),
            ),
            selected_block_id: Some(call_function_view.selected_block_id.into()),
        }
    }
}

impl From<CliCallFunctionView> for CallFunctionView {
    fn from(item: CliCallFunctionView) -> Self {
        let method_name: String = match item.method_name {
            Some(cli_method_name) => cli_method_name,
            None => CallFunctionView::input_method_name(),
        };
        let function_args: Vec<u8> = match item.function_args {
            Some(cli_args) => cli_args.into_bytes(),
            None => CallFunctionView::input_function_args(),
        };
        let selected_block_id: super::block_id::BlockId = match item.selected_block_id {
            Some(cli_block_id) => cli_block_id.into(),
            None => super::block_id::BlockId::choose_block_id(),
        };
        Self {
            method_name,
            function_args,
            selected_block_id,
        }
    }
}

impl CallFunctionView {
    fn input_method_name() -> String {
        println!();
        Input::new()
            .with_prompt("Enter a method name")
            .interact_text()
            .unwrap()
    }

    fn input_function_args() -> Vec<u8> {
        println!();
        let input: String = Input::new()
            .with_prompt("Enter args for function")
            .interact_text()
            .unwrap();
        input.into_bytes()
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
                self.function_args,
            )
            .await
    }
}
