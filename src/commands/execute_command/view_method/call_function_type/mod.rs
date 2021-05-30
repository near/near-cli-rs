use dialoguer::Input;

/// Call view function
#[derive(Debug, Default, clap::Clap)]
pub struct CliCallFunctionView {
    method_name: Option<String>,
    args: Option<String>,
    #[clap(subcommand)]
    selected_block_id: Option<super::block_id::CliBlockId>,
}

#[derive(Debug)]
pub struct CallFunctionView {
    method_name: String,
    args: Vec<u8>,
    selected_block_id: super::block_id::BlockId,
}

impl From<CliCallFunctionView> for CallFunctionView {
    fn from(item: CliCallFunctionView) -> Self {
        let method_name: String = match item.method_name {
            Some(cli_method_name) => cli_method_name,
            None => CallFunctionView::input_method_name(),
        };
        let args: Vec<u8> = match item.args {
            Some(cli_args) => cli_args.into_bytes(),
            None => CallFunctionView::input_args(),
        };
        let selected_block_id: super::block_id::BlockId = match item.selected_block_id {
            Some(cli_block_id) => cli_block_id.into(),
            None => super::block_id::BlockId::choose_block_id(),
        };
        Self {
            method_name,
            args,
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

    fn input_args() -> Vec<u8> {
        println!();
        let input: String = Input::new()
            .with_prompt("Enter args for function")
            .interact_text()
            .unwrap();
        input.into_bytes()
    }

    pub async fn process(
        self,
        selected_server_url: url::Url,
        contract_account_id: String,
    ) -> crate::CliResult {
        self.selected_block_id
            .process(
                contract_account_id,
                selected_server_url,
                self.method_name,
                self.args,
            )
            .await
    }
}
