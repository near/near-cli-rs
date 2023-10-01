use std::str::FromStr;

use color_eyre::eyre::WrapErr;
use inquire::Text;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::UpdateAccountProfileContext)]
#[interactive_clap(output_context = JsonArgsContext)]
pub struct JsonArgs {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter valid JSON arguments (e.g. {\"token_id\": \"42\"})":
    data: String,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: super::super::sign_as::Signer,
}

#[derive(Clone)]
pub struct JsonArgsContext(super::ArgsContext);

impl JsonArgsContext {
    pub fn from_previous_context(
        previous_context: super::super::UpdateAccountProfileContext,
        scope: &<JsonArgs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(super::ArgsContext {
            global_context: previous_context.global_context,
            get_contract_account_id: previous_context.get_contract_account_id,
            account_id: previous_context.account_id,
            data: serde_json::Value::from_str(&scope.data)
                .wrap_err("Data not in JSON format!")?
                .to_string()
                .into_bytes(),
        }))
    }
}

impl From<JsonArgsContext> for super::ArgsContext {
    fn from(item: JsonArgsContext) -> Self {
        item.0
    }
}

impl JsonArgs {
    fn input_data(
        _context: &super::super::UpdateAccountProfileContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        loop {
            let data =
                Text::new("Enter valid JSON arguments (e.g. {\"token_id\": \"42\"}):").prompt()?;
            if serde_json::Value::from_str(&data).is_ok() {
                return Ok(Some(data));
            }
            eprintln!("Data not in JSON format!")
        }
    }
}
