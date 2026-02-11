mod constants;

mod account;
mod contract;
mod deprecated;
mod keys;
mod transactions;

#[derive(Debug, Clone, clap::Parser)]
/// Legacy CLI commands are only supported at best-effort
pub enum JsCmd {
    #[command(alias("create"))]
    CreateAccount(self::account::create::CreateAccountArgs),
    #[command(alias("delete"))]
    DeleteAccount(self::account::delete::DeleteAccountArgs),
    #[command(alias("import-account"))]
    Login(self::account::login::LoginArgs),
    State(self::account::state::StateArgs),

    Call(self::contract::call::CallArgs),
    Deploy(self::contract::deploy::DeployArgs),
    #[command(alias("storage"))]
    ViewState(self::contract::storage::ViewStateArgs),
    View(self::contract::view::ViewArgs),

    AddKey(self::keys::add::AddKeyArgs),
    DeleteKey(self::keys::delete::DeleteKeyArgs),
    #[command(alias("keys"))]
    ListKeys(self::keys::list::KeysArgs),

    #[command(alias("send-near"))]
    Send(self::transactions::send::SendArgs),
    TxStatus(self::transactions::status::TxStatusArgs),

    Validators(self::deprecated::ValidatorsArgs),
    #[command(alias("validator-stake"))]
    Stake(self::deprecated::StakeArgs),
}

impl JsCmd {
    pub fn rust_command_generation(&self) -> Vec<String> {
        let network = std::env::var("NEAR_NETWORK")
            .or_else(|_| std::env::var("NEAR_ENV"))
            .unwrap_or_else(|_| "testnet".to_owned());

        match self {
            Self::CreateAccount(args) => args.to_cli_args(network),
            Self::DeleteAccount(args) => args.to_cli_args(network),
            Self::Login(args) => args.to_cli_args(network),
            Self::State(args) => args.to_cli_args(network),

            Self::Call(args) => args.to_cli_args(network),
            Self::Deploy(args) => args.to_cli_args(network),
            Self::ViewState(args) => args.to_cli_args(network),
            Self::View(args) => args.to_cli_args(network),

            Self::AddKey(args) => args.to_cli_args(network),
            Self::DeleteKey(args) => args.to_cli_args(network),
            Self::ListKeys(args) => args.to_cli_args(network),

            Self::Send(args) => args.to_cli_args(network),
            Self::TxStatus(args) => args.to_cli_args(network),

            Self::Validators(args) => args.to_cli_args(network),
            Self::Stake(args) => args.to_cli_args(network),
        }
    }
}
