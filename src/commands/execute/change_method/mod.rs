use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

pub mod call_function_type;
mod operation_mode;
mod receiver;
mod sender;


/// формирование вызова метода изменения
#[derive(Debug, Default, clap::Clap)]
pub struct CliChangeMethod {
    #[clap(subcommand)]
    call: Option<CliCallFunction>
}

#[derive(Debug)]
pub struct ChangeMethod {
    call: CallFunction
}

impl From<CliChangeMethod> for ChangeMethod {
    fn from(item: CliChangeMethod) -> Self {
        let call = match item.call {
            Some(cli_call_function) => CallFunction::from(cli_call_function),
            None => CallFunction::choose_call_function()
        };
        Self { call }
    }
}

impl ChangeMethod {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.call.process(prepopulated_unsigned_transaction).await
    }
}

#[derive(Debug, clap::Clap)]
pub enum CliCallFunction {
    /// вызов метода изменения
    Call(self::call_function_type::CliCallFunctionAction),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum CallFunction {
    #[strum_discriminants(strum(message = "Call function"))]
    Call(self::call_function_type::CallFunctionAction),
}

impl From<CliCallFunction> for CallFunction {
    fn from(item: CliCallFunction) -> Self {
        match item {
            CliCallFunction::Call(cli_call_function_action) => {
                CallFunction::Call(cli_call_function_action.into())
            }
        }
    }
}

impl CallFunction {
    fn choose_call_function() -> Self {
        println!();
        let variants = CallFunctionDiscriminants::iter().collect::<Vec<_>>();
        let commands = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose your action")
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
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self {
            Self::Call(call_function_action) => call_function_action.process(prepopulated_unsigned_transaction).await,
        }
    }
}
