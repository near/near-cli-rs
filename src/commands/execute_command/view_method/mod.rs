use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod call_function_type;
pub mod operation_mode;
mod receiver;


#[derive(Debug, clap::Clap)]
pub enum CliCallFunction {
    /// вызов метода просмотра
    Call(self::call_function_type::CliCallFunctionView),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum CallFunction {
    #[strum_discriminants(strum(message = "Call function"))]
    Call(self::call_function_type::CallFunctionView),
}

impl From<CliCallFunction> for CallFunction {
    fn from(item: CliCallFunction) -> Self {
        match item {
            CliCallFunction::Call(cli_call_function_view) => {
                CallFunction::Call(cli_call_function_view.into())
            }
        }
    }
}

impl CallFunction {
    pub fn choose_call_function() -> Self {
        println!();
        let variants = CallFunctionDiscriminants::iter().collect::<Vec<_>>();
        let commands = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Call your function")
            .items(&commands)
            .default(0)
            .interact()
            .unwrap();
        let cli_call = match variants[selection] {
            CallFunctionDiscriminants::Call => {
                CliCallFunction::Call(Default::default())
            }
        };
        Self::from(cli_call)
    }

    pub async fn process(
        self,
        selected_server_url: url::Url,
        contract_account_id: String,
    ) -> crate::CliResult {
        match self {
            Self::Call(call_function_action) => {
                call_function_action.process(selected_server_url, contract_account_id).await
            }
        }
    }
}
