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
    ) -> Result<(Vec<String>, String), String> {
        let network = std::env::var("NEAR_NETWORK")
            .unwrap_or_else(|_| std::env::var("NEAR_ENV").unwrap_or_else(|_| "testnet".to_owned()));
        let message = "The command you tried to run is deprecated in the new NEAR CLI, but we tried our best to match the old command with the new syntax, try it instead:".to_string();
        let validator_extension_message = "The command you tried to run has been moved into its own CLI extension called near-validator.\nPlease, follow the installation instructions here: https://github.com/near-cli-rs/near-validator-cli-rs/blob/master/README.md".to_string();

        match self {
            Self::CreateAccount(args) => Ok((args.to_cli_args(network), message)),
            Self::DeleteAccount(args) => Ok((args.to_cli_args(network), message)),
            Self::Login(args) => Ok((args.to_cli_args(network).into(), message)),
            Self::State(args) => Ok((args.to_cli_args(network), message)),

            Self::Call(args) => Ok((args.to_cli_args(network), message)),
            Self::Deploy(args) => Ok((args.to_cli_args(network), message)),
            Self::ViewState(args) => Ok((args.to_cli_args(network), message)),
            Self::View(args) => Ok((args.to_cli_args(network), message)),

            Self::AddKey(args) => Ok((args.to_cli_args(network), message)),
            Self::DeleteKey(args) => Ok((args.to_cli_args(network), message)),
            Self::ListKeys(args) => Ok((args.to_cli_args(network), message)),

            Self::Send(args) => Ok((args.to_cli_args(network), message)),
            Self::TxStatus(args) => Ok((args.to_cli_args(network), message)),

            Self::Stake(_args) => Ok((vec![], validator_extension_message)),
            Self::Validators(_args) => Ok((vec![], validator_extension_message)),
        }
    }
}
