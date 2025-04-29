use inquire::{Select, Text};
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod as_read_only;
mod as_transaction;
pub mod call_function_args_type;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct CallFunctionCommands {
    #[interactive_clap(subcommand)]
    function_call_actions: CallFunctionActions,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Сhoose action for account:
pub enum CallFunctionActions {
    #[strum_discriminants(strum(message = "as-read-only    - Calling a view method"))]
    /// Calling a view method
    AsReadOnly(self::as_read_only::CallFunctionView),
    #[strum_discriminants(strum(message = "as-transaction  - Calling a change method"))]
    /// Calling a change method
    AsTransaction(self::as_transaction::CallFunction),
}

pub fn input_call_function_name(
    global_context: &crate::GlobalContext,
    contract_account_id: &near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<Option<String>> {
    input_function_name(
        global_context,
        contract_account_id,
        near_abi::AbiFunctionKind::Call,
        "Select the as-transaction function for your contract:",
    )
}

pub fn input_view_function_name(
    global_context: &crate::GlobalContext,
    contract_account_id: &near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<Option<String>> {
    input_function_name(
        global_context,
        contract_account_id,
        near_abi::AbiFunctionKind::View,
        "Select the viewing function for your contract:",
    )
}

fn input_function_name(
    global_context: &crate::GlobalContext,
    contract_account_id: &near_primitives::types::AccountId,
    function_kind: near_abi::AbiFunctionKind,
    message: &str,
) -> color_eyre::eyre::Result<Option<String>> {
    let network_config = crate::common::find_network_where_account_exist(
        global_context,
        contract_account_id.clone(),
    )?;

    if let Some(network_config) = network_config {
        let json_rpc_client = network_config.json_rpc_client();
        if let Ok(contract_abi) =
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(super::inspect::get_contract_abi(
                    &json_rpc_client,
                    &near_primitives::types::Finality::Final.into(),
                    contract_account_id,
                ))
        {
            let function_names = contract_abi
                .body
                .functions
                .into_iter()
                .filter(|function| function_kind == function.kind)
                .map(|function| function.name)
                .collect::<Vec<String>>();
            if !function_names.is_empty() {
                return Ok(Some(
                    Select::new(message, function_names).prompt()?.to_string(),
                ));
            }
        }
    }

    Ok(Some(
        Text::new("What is the name of the function?").prompt()?,
    ))
}
