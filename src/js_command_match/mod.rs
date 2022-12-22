mod create_account;
mod delete;
mod keys;
mod state;

#[derive(Debug, Clone, clap::Parser)]
pub enum JsCmd {
    CreateAccount(self::create_account::CreateAccountArgs),
    State(self::state::StateArgs),
    Delete(self::delete::DeleteArgs),
    Keys(self::keys::KeysArgs),
}

impl JsCmd {
    pub fn rust_command_generation(&self) -> Vec<String> {
        match self {
            Self::CreateAccount(create_account_args) => create_account_args.to_cli_args(),
            Self::State(state_args) => state_args.to_cli_args(),
            Self::Delete(delete_args) => delete_args.to_cli_args(),
            Self::Keys(keys_args) => keys_args.to_cli_args(),
        }
    }
}
