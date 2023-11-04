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
    pub fn rust_command_generation(
        &self,
    ) -> color_eyre::eyre::Result<(Vec<String>, String), String> {
        //NEAR_ENV=testnet default
        let network_config = std::env::var("NEAR_ENV").unwrap_or_else(|_| "testnet".to_owned());
        let message = "The command you tried to run is deprecated in the new NEAR CLI, but we tried our best to match the old command with the new syntax, try it instead:".to_string();
        let near_validator_extension_message = "The command you tried to run has been moved into its own CLI extension called near-validator.\nPlease, follow the installation instructions here: https://github.com/near-cli-rs/near-validator-cli-rs/blob/master/README.md\nThen run the following command:".to_string();
        match self {
            Self::CreateAccount(create_account_args) => Ok((create_account_args.to_cli_args(network_config), message)),
            Self::State(state_args) => Ok((state_args.to_cli_args(network_config), message)),
            Self::Delete(delete_args) => Ok((delete_args.to_cli_args(network_config), message)),
            Self::Keys(keys_args) => Ok((keys_args.to_cli_args(network_config), message)),
            Self::TxStatus(tx_status_args) => Ok((tx_status_args.to_cli_args(network_config), message)),
            Self::Deploy(deploy_args) => Ok((deploy_args.to_cli_args(network_config), message)),
            Self::DevDeploy(_) => Err("`dev-deploy` command is not implemented, yet. It will be implemented in a dev extension. Meanwhile, consider using the old CLI or a standalone implementation: https://github.com/frolvanya/dev-deploy".to_string()),
            Self::Call(call_args) => Ok((call_args.to_cli_args(network_config), message)),
            Self::View(view_args) => Ok((view_args.to_cli_args(network_config), message)),
            Self::ViewState(view_state_args) => Ok((view_state_args.to_cli_args(network_config), message)),
            Self::Send(send_args) => Ok((send_args.to_cli_args(network_config), message)),
            Self::Clean(_) => Err("`clean` command is not implemented, yet. It will be implemented in a dev extension. Meanwhile, keep using the old CLI.".to_string()),
            Self::Stake(stake_args) => Ok((stake_args.to_cli_args(network_config), near_validator_extension_message)),
            Self::Login(login_args) => Ok((login_args.to_cli_args(network_config), message)),
            Self::Repl(_) => Err("`repl` command is not implemented. Use shell scripting for the new CLI.".to_string()),
            Self::GenerateKey(generate_key_args) => {
                match generate_key_args.to_cli_args(network_config){
                    Ok(res) => Ok((res, message)),
                    Err(err) => Err(err.to_string())
                }
            },
            Self::AddKey(add_key_args) => Ok((add_key_args.to_cli_args(network_config), message)),
            Self::DeleteKey(delete_key_args) => Ok((delete_key_args.to_cli_args(network_config), message)),
            Self::Validators(validators_args) => Ok((validators_args.to_cli_args(network_config), near_validator_extension_message)),
            Self::Proposals(proposals_args) => Ok((proposals_args.to_cli_args(network_config), near_validator_extension_message)),
            Self::EvmCall(_) => Err("`evm-call` command is not implemented, yet. It will be implemented in an evm extension. Meanwhile, keep using the old CLI.".to_string()),
            Self::EvmDevInit(_) => Err("`evm-dev-init` command is not implemented, yet. It will be implemented in an evm extension. Meanwhile, keep using the old CLI.".to_string()),
            Self::EvmView(_) => Err("`evm-view` command is not implemented, yet. It will be implemented in an evm extension. Meanwhile, keep using the old CLI.".to_string()),
            Self::SetApiKey(set_api_key_args) => {
                match set_api_key_args.to_cli_args(network_config){
                    Ok(res) => Ok((res, message)),
                    Err(err) => Err(err.to_string())
                }
            },
            Self::Js(_) => Err("`js` command is not implemented. Use shell scripting for the new CLI.".to_string()),
        }
    }
}
