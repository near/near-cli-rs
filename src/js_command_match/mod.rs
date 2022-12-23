mod call;
mod create_account;
mod delete;
mod deploy;
mod dev_deploy;
mod keys;
mod send;
mod state;
mod tx_status;
mod view;
mod view_state;

#[derive(Debug, Clone, clap::Parser)]
pub enum JsCmd {
    CreateAccount(self::create_account::CreateAccountArgs),
    State(self::state::StateArgs),
    Delete(self::delete::DeleteArgs),
    Keys(self::keys::KeysArgs),
    TxStatus(self::tx_status::TxStatusArgs),
    Deploy(self::deploy::DeployArgs),
    DevDeploy(self::dev_deploy::DevDeployArgs),
    Call(self::call::CallArgs),
    View(self::view::ViewArgs),
    ViewState(self::view_state::ViewStateArgs),
    Send(self::send::SendArgs),
}

impl JsCmd {
    pub fn rust_command_generation(&self) -> color_eyre::eyre::Result<Vec<String>, String> {
        match self {
            Self::CreateAccount(create_account_args) => Ok(create_account_args.to_cli_args()),
            Self::State(state_args) => Ok(state_args.to_cli_args()),
            Self::Delete(delete_args) => Ok(delete_args.to_cli_args()),
            Self::Keys(keys_args) => Ok(keys_args.to_cli_args()),
            Self::TxStatus(tx_status_args) => Ok(tx_status_args.to_cli_args()),
            Self::Deploy(deploy_args) => Ok(deploy_args.to_cli_args()),
            Self::DevDeploy(_) => Err("We plan to implement it in dev extension. Here is a standalone implementation: https://github.com/frolvanya/dev-deploy".to_string()),
            Self::Call(call_args) => Ok(call_args.to_cli_args()),
            Self::View(view_args) => Ok(view_args.to_cli_args()),
            Self::ViewState(_) => Err("We plan to implement it in dev extension".to_string()),
            Self::Send(send_args) => Ok(send_args.to_cli_args()),
        }
    }
}
