mod constants;

mod account;
mod contract;
mod keys;
mod transactions;
mod validators;

#[derive(Debug, Clone, clap::Parser)]
/// Legacy CLI commands are only supported at best-effort
pub enum JsCmd {
    #[clap(alias("create"))]
    CreateAccount(self::account::create::CreateAccountArgs),
    #[clap(alias("delete"))]
    DeleteAccount(self::account::delete::DeleteAccountArgs),
    #[clap(alias("import-account"))]
    Login(self::account::login::LoginArgs),
    State(self::account::state::StateArgs),

    Call(self::contract::call::CallArgs),
    Deploy(self::contract::deploy::DeployArgs),
    #[clap(alias("storage"))]
    ViewState(self::contract::storage::ViewStateArgs),
    View(self::contract::view::ViewArgs),

    AddKey(self::keys::add::AddKeyArgs),
    DeleteKey(self::keys::delete::DeleteKeyArgs),
    #[clap(alias("keys"))]
    ListKeys(self::keys::list::KeysArgs),

    #[clap(alias("send-near"))]
    Send(self::transactions::send::SendArgs),
    TxStatus(self::transactions::status::TxStatusArgs),

    #[clap(alias("validator-stake"))]
    Stake(self::validators::StakeArgs),
    Validators(self::validators::ValidatorsArgs),
}

impl JsCmd {
    pub fn rust_command_generation(&self) -> Result<(Vec<String>, String), String> {
        let network = std::env::var("NEAR_NETWORK")
            .or_else(|_| std::env::var("NEAR_ENV"))
            .unwrap_or_else(|_| "testnet".to_owned());
        let message = "The command you tried to run is deprecated in the new NEAR CLI, but we tried our best to match the old command with the new syntax, try it instead:".to_string();
        let validator_extension_message = "The command you tried to run has been moved into its own CLI extension called near-validator.\nPlease, follow the installation instructions here: https://github.com/near-cli-rs/near-validator-cli-rs/blob/master/README.md".to_string();

        match self {
            Self::CreateAccount(args) => Ok((args.to_cli_args(network), message)),
            Self::DeleteAccount(args) => Ok((args.to_cli_args(network), message)),
            Self::Login(args) => Ok((args.to_cli_args(network), message)),
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
