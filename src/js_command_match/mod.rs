mod call;
mod create_account;
mod delete;
mod deploy;
mod keys;
mod state;
mod tx_status;

#[derive(Debug, Clone, clap::Parser)]
pub enum JsCmd {
    CreateAccount(self::create_account::CreateAccountArgs),
    State(self::state::StateArgs),
    Delete(self::delete::DeleteArgs),
    Keys(self::keys::KeysArgs),
    TxStatus(self::tx_status::TxStatusArgs),
    Deploy(self::deploy::DeployArgs),
    DevDeploy,
    Call(self::call::CallArgs),
}

impl JsCmd {
    pub fn rust_command_generation(&self) -> Vec<String> {
        match self {
            Self::CreateAccount(create_account_args) => create_account_args.to_cli_args(),
            Self::State(state_args) => state_args.to_cli_args(),
            Self::Delete(delete_args) => delete_args.to_cli_args(),
            Self::Keys(keys_args) => keys_args.to_cli_args(),
            Self::TxStatus(tx_status_args) => tx_status_args.to_cli_args(),
            Self::Deploy(deploy_args) => deploy_args.to_cli_args(),
            Self::DevDeploy => todo!("We plan to implement it in dev extension. Here is a standalone implementation: https://github.com/frolvanya/dev-deploy"),
            Self::Call(call_args) => call_args.to_cli_args(),
        }
    }
}
