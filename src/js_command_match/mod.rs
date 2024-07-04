mod constants;

mod account;
mod contract;
mod keys;
mod transactions;
mod validators;

#[derive(Debug, Clone, clap::Parser)]
/// Legacy CLI commands are only supported at best-effort
pub enum JsCmd {
    CreateAccount(self::account::create::CreateAccountArgs),
    DeleteAccount(self::account::delete::DeleteAccountArgs),
    Login(self::account::login::LoginArgs),
    State(self::account::state::StateArgs),

    Call(self::contract::call::CallArgs),
    Deploy(self::contract::deploy::DeployArgs),
    ViewState(self::contract::storage::ViewStateArgs),
    View(self::contract::view::ViewArgs),

    AddKey(self::keys::add::AddKeyArgs),
    DeleteKey(self::keys::delete::DeleteKeyArgs),
    ListKeys(self::keys::list::KeysArgs),

    Send(self::transactions::send::SendArgs),
    TxStatus(self::transactions::status::TxStatusArgs),

    Stake(self::validators::stake::StakeArgs),
    Validators(self::validators::validators::ValidatorsArgs),
}

impl JsCmd {
    pub fn rust_command_generation(
        &self,
    ) -> color_eyre::eyre::Result<(Vec<String>, String), String> {
        eprintln!("{:?}", self);

        let network_config = std::env::var("NEAR_NETWORK")
            .unwrap_or_else(|_| std::env::var("NEAR_ENV").unwrap_or_else(|_| "testnet".to_owned()));
        let message = "The command you tried to run is deprecated in the new NEAR CLI, but we tried our best to match the old command with the new syntax, try it instead:".to_string();
        let near_validator_extension_message = "The command you tried to run has been moved into its own CLI extension called near-validator.\nPlease, follow the installation instructions here: https://github.com/near-cli-rs/near-validator-cli-rs/blob/master/README.md".to_string();

        match self {
            Self::CreateAccount(create_account_args) => {
                Ok((create_account_args.to_cli_args(network_config), message))
            }
            Self::DeleteAccount(delete_account_args) => {
                Ok((delete_account_args.to_cli_args(network_config), message))
            }
            Self::Login(login_args) => Ok((login_args.to_cli_args(network_config).into(), message)),
            Self::State(state_args) => Ok((state_args.to_cli_args(network_config), message)),

            Self::Call(call_args) => Ok((call_args.to_cli_args(network_config), message)),
            Self::Deploy(deploy_args) => Ok((deploy_args.to_cli_args(network_config), message)),
            Self::ViewState(view_state_args) => {
                Ok((view_state_args.to_cli_args(network_config), message))
            }
            Self::View(view_args) => Ok((view_args.to_cli_args(network_config), message)),

            Self::AddKey(add_key_args) => Ok((add_key_args.to_cli_args(network_config), message)),
            Self::DeleteKey(delete_key_args) => {
                Ok((delete_key_args.to_cli_args(network_config), message))
            }
            Self::ListKeys(keys_args) => Ok((keys_args.to_cli_args(network_config), message)),

            Self::Send(send_args) => Ok((send_args.to_cli_args(network_config), message)),
            Self::TxStatus(tx_status_args) => {
                Ok((tx_status_args.to_cli_args(network_config), message))
            }

            Self::Stake(_stake_args) => Ok((
                vec![],
                near_validator_extension_message,
            )),
            Self::Validators(_validators_args) => Ok((
                vec![],
                near_validator_extension_message,
            )),
        }
    }
}
