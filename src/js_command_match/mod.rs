mod create_account;
mod delete;

#[derive(Debug, Clone, clap::Parser)]
pub enum JsCmd {
    CreateAccount(self::create_account::CreateAccountArgs),
    Delete(self::delete::DeleteArgs),
}

impl JsCmd {
    pub fn rust_command_generation(&self) -> Vec<String> {
        match self {
            Self::CreateAccount(create_account_args) => create_account_args.to_cli_args(),
            Self::Delete(delete_args) => delete_args.to_cli_args(),
        }
    }
}
