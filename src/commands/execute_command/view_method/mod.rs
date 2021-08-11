use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod block_id;
mod call_function_type;
pub mod operation_mode;
mod receiver;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliCallFunction {
    /// Call view function
    Call(self::call_function_type::CliCallFunctionView),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum CallFunction {
    #[strum_discriminants(strum(message = "Call function"))]
    Call(self::call_function_type::CallFunctionView),
}

impl CliCallFunction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::Call(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("call".to_owned());
                args
            }
        }
    }
}

impl From<CallFunction> for CliCallFunction {
    fn from(call_function: CallFunction) -> Self {
        match call_function {
            CallFunction::Call(call_function_action) => Self::Call(call_function_action.into()),
        }
    }
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
            CallFunctionDiscriminants::Call => CliCallFunction::Call(Default::default()),
        };
        Self::from(cli_call)
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
        contract_account_id: near_primitives::types::AccountId,
    ) -> crate::CliResult {
        match self {
            Self::Call(call_function_action) => {
                call_function_action
                    .process(network_connection_config, contract_account_id)
                    .await
            }
        }
    }
}
