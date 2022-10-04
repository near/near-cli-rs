use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///How do you want to pass the function call arguments?
pub enum CallFunctionArgs {
    #[strum_discriminants(strum(
        message = "json-args    - Valid JSON arguments (e.g. {\"token_id\": \"42\"})"
    ))]
    ///Valid JSON arguments (e.g. {"token_id": "42"})
    JsonArgs,
    #[strum_discriminants(strum(message = "text-args    - Arbitrary text arguments"))]
    ///Arbitrary text arguments
    TextArgs(TextArgs),
    #[strum_discriminants(strum(message = "base64-args  - Base64-encoded string (e.g. e30=)"))]
    ///Base64-encoded string (e.g. e30=)
    Base64Args,
    #[strum_discriminants(strum(
        message = "file-args    - Read from file (e.g. reusable JSON or binary data)"
    ))]
    ///Read from file (e.g. reusable JSON or binary data)
    FileArgs,
}

impl CallFunctionArgs {
    pub async fn process(
        &self,
        config: crate::config::Config,
        function_call_action_optional: Option<super::as_transaction::FunctionCallAction>,
    ) -> crate::CliResult {
        if let Some(function_call_action) = function_call_action_optional {
            match self {
                Self::JsonArgs => todo!(),
                Self::TextArgs(text_args) => text_args.process(config, function_call_action).await,
                Self::Base64Args => todo!(),
                Self::FileArgs => todo!(),
            }
        } else {
            match self {
                Self::JsonArgs => todo!(),
                Self::TextArgs(text_args) => todo!(),
                Self::Base64Args => todo!(),
                Self::FileArgs => todo!(),
            }
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct TextArgs {
    ///Enter arguments to this function
    function_args: String,
    #[interactive_clap(named_arg)]
    ///Enter gas for function call
    prepaid_gas: super::as_transaction::PrepaidGas,
}

impl TextArgs {
    pub async fn process(
        &self,
        config: crate::config::Config,
        function_call_action: super::as_transaction::FunctionCallAction,
    ) -> crate::CliResult {
        let function_call_action = super::as_transaction::FunctionCallAction {
            function_args: self.function_args.clone(),
            ..function_call_action
        };
        self.prepaid_gas.process(config, function_call_action).await
    }
}
