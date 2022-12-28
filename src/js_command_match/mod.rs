mod call;
mod clean;
mod create_account;
mod delete;
mod deploy;
mod dev_deploy;
mod keys;
mod login;
mod send;
mod stake;
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
    Clean(self::clean::CleanArgs),
    Stake(self::stake::StakeArgs),
    Login(self::login::LoginArgs),
}

impl JsCmd {
    pub fn rust_command_generation(&self) -> color_eyre::eyre::Result<Vec<String>, String> {
        //NEAR_ENV=testnet default
        let network_config = std::env::var("NEAR_ENV").unwrap_or_else(|_| "testnet".to_owned());
        match self {
            Self::CreateAccount(create_account_args) => Ok(create_account_args.to_cli_args(network_config)),
            Self::State(state_args) => Ok(state_args.to_cli_args(network_config)),
            Self::Delete(delete_args) => Ok(delete_args.to_cli_args(network_config)),
            Self::Keys(keys_args) => Ok(keys_args.to_cli_args(network_config)),
            Self::TxStatus(tx_status_args) => Ok(tx_status_args.to_cli_args(network_config)),
            Self::Deploy(deploy_args) => Ok(deploy_args.to_cli_args(network_config)),
            Self::DevDeploy(_) => Err("We plan to implement it in dev extension. Here is a standalone implementation: https://github.com/frolvanya/dev-deploy".to_string()),
            Self::Call(call_args) => Ok(call_args.to_cli_args(network_config)),
            Self::View(view_args) => Ok(view_args.to_cli_args(network_config)),
            Self::ViewState(_) => Err("We plan to implement it in dev extension".to_string()),
            Self::Send(send_args) => Ok(send_args.to_cli_args(network_config)),
            Self::Clean(_) => Err("Potentially will be implemented in dev extension.".to_string()),
            Self::Stake(_) => Err("We plan to implement it in validators extension".to_string()),
            Self::Login(login_args) => Ok(login_args.to_cli_args(network_config)),
        }
    }
}
