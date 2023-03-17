use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod call_function;
// mod deploy;
// mod download_wasm;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct ContractCommands {
    #[interactive_clap(subcommand)]
    contract_actions: ContractActions,
}

impl ContractCommands {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        self.contract_actions.process(config).await
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///Сhoose action for account
pub enum ContractActions {
    #[strum_discriminants(strum(
        message = "call-function   - Execute function (contract method)"
    ))]
    ///Execute function (contract method)
    CallFunction(self::call_function::CallFunctionCommands),
    // #[strum_discriminants(strum(message = "deploy          - Add a new contract code"))]
    // ///Add a contract code
    // Deploy(self::deploy::Contract),
    // #[strum_discriminants(strum(message = "download-wasm   - Download wasm"))]
    // ///Download wasm
    // DownloadWasm(self::download_wasm::ContractAccount),
}

impl ContractActions {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        match self {
            Self::CallFunction(call_function_commands) => {
                call_function_commands.process(config).await
            }
            // Self::Deploy(contract) => contract.process(config).await,
            // Self::DownloadWasm(download_contract) => download_contract.process(config).await,
        }
    }
}
