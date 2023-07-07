mod add_key;
mod call;
mod clean;
mod create_account;
mod delete;
mod delete_key;
mod deploy;
mod dev_deploy;
mod evm_call;
mod evm_dev_init;
mod evm_view;
mod generate_key;
mod js;
mod keys;
mod login;
mod proposals;
mod repl;
mod send;
mod set_api_key;
mod stake;
mod state;
mod tx_status;
mod validators;
mod view;
mod view_state;

#[derive(Debug, Clone, clap::Parser)]
/// Legacy CLI commands are only supported at best-effort
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
    Repl(self::repl::ReplArgs),
    GenerateKey(self::generate_key::GenerateKeyArgs),
    AddKey(self::add_key::AddKeyArgs),
    DeleteKey(self::delete_key::DeleteKeyArgs),
    Validators(self::validators::ValidatorsArgs),
    Proposals(self::proposals::ProposalsArgs),
    EvmCall(self::evm_call::EvmCallArgs),
    EvmDevInit(self::evm_dev_init::EvmDevInitArgs),
    EvmView(self::evm_view::EvmViewArgs),
    SetApiKey(self::set_api_key::SetApiKeyArgs),
    Js(self::js::JsArgs),
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
            Self::DevDeploy(_) => Err("`dev-deploy` command is not implemented, yet. It will be implemented in a dev extension. Meanwhile, consider using the old CLI or a standalone implementation: https://github.com/frolvanya/dev-deploy".to_string()),
            Self::Call(call_args) => Ok(call_args.to_cli_args(network_config)),
            Self::View(view_args) => Ok(view_args.to_cli_args(network_config)),
            Self::ViewState(_) => Err("`view-state` command is not implemented, yet. It will be implemented in a dev extension. Meanwhile, keep using the old CLI.".to_string()),
            Self::Send(send_args) => Ok(send_args.to_cli_args(network_config)),
            Self::Clean(_) => Err("`clean` command is not implemented, yet. It will be implemented in a dev extension. Meanwhile, keep using the old CLI.".to_string()),
            Self::Stake(_) => Err("`stake` command is not implemented, yet. It will be implemented in a validators extension. Meanwhile, keep using the old CLI.".to_string()),
            Self::Login(login_args) => Ok(login_args.to_cli_args(network_config)),
            Self::Repl(_) => Err("`repl` command is not implemented. Use shell scripting for the new CLI.".to_string()),
            Self::GenerateKey(generate_key_args) => {
                match generate_key_args.to_cli_args(network_config){
                    Ok(res) => Ok(res),
                    Err(err) => Err(err.to_string())
                }
            },
            Self::AddKey(add_key_args) => Ok(add_key_args.to_cli_args(network_config)),
            Self::DeleteKey(delete_key_args) => Ok(delete_key_args.to_cli_args(network_config)),
            Self::Validators(validators) => Ok(validators.to_cli_args(network_config)),
            Self::Proposals(_) => Err("`proposals` command is not implemented, yet. It will be implemented in a validators extension. Meanwhile, keep using the old CLI.".to_string()),
            Self::EvmCall(_) => Err("`evm-call` command is not implemented, yet. It will be implemented in an evm extension. Meanwhile, keep using the old CLI.".to_string()),
            Self::EvmDevInit(_) => Err("`evm-dev-init` command is not implemented, yet. It will be implemented in an evm extension. Meanwhile, keep using the old CLI.".to_string()),
            Self::EvmView(_) => Err("`evm-view` command is not implemented, yet. It will be implemented in an evm extension. Meanwhile, keep using the old CLI.".to_string()),
            Self::SetApiKey(set_api_key_args) => {
                match set_api_key_args.to_cli_args(network_config){
                    Ok(res) => Ok(res),
                    Err(err) => Err(err.to_string())
                }
            },
            Self::Js(_) => Err("`js` command is not implemented. Use shell scripting for the new CLI.".to_string()),
        }
    }
}
