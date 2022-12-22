mod delete;

#[derive(Debug, Clone, clap::Parser)]
pub enum JsCmd {
    Delete(self::delete::DeleteArgs),
}

impl JsCmd {
    pub fn rust_command_generation(&self) -> Vec<String> {
        match self {
            Self::Delete(delete_args) => delete_args.to_cli_args(),
        }
    }
}
