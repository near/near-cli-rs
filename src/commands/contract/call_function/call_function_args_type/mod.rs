use std::str::FromStr;

use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

#[derive(Debug, EnumDiscriminants, Clone, clap::ArgEnum)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///How do you want to pass the function call arguments?
pub enum FunctionArgsType {
    #[strum_discriminants(strum(
        message = "json-args    - Valid JSON arguments (e.g. {\"token_id\": \"42\"})"
    ))]
    ///Valid JSON arguments (e.g. {"token_id": "42"})
    JsonArgs,
    #[strum_discriminants(strum(message = "text-args    - Arbitrary text arguments"))]
    ///Arbitrary text arguments
    TextArgs,
    #[strum_discriminants(strum(message = "base64-args  - Base64-encoded string (e.g. e30=)"))]
    ///Base64-encoded string (e.g. e30=)
    Base64Args,
    #[strum_discriminants(strum(
        message = "file-args    - Read from file (e.g. reusable JSON or binary data)"
    ))]
    ///Read from file (e.g. reusable JSON or binary data)
    FileArgs,
}

impl interactive_clap::ToCli for FunctionArgsType {
    type CliVariant = FunctionArgsType;
}

impl std::str::FromStr for FunctionArgsType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json-args" => Ok(Self::JsonArgs),
            "text-args" => Ok(Self::TextArgs),
            "base64-args" => Ok(Self::Base64Args),
            "file-args" => Ok(Self::FileArgs),
            _ => Err("FunctionArgsType: incorrect value entered".to_string()),
        }
    }
}

impl std::fmt::Display for FunctionArgsType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::JsonArgs => write!(f, "json-args"),
            Self::TextArgs => write!(f, "text-args"),
            Self::Base64Args => write!(f, "base64-args"),
            Self::FileArgs => write!(f, "file-args"),
        }
    }
}

pub fn input_function_args_type() -> color_eyre::eyre::Result<FunctionArgsType> {
    let variants = FunctionArgsTypeDiscriminants::iter().collect::<Vec<_>>();
    let function_args_types = variants
        .iter()
        .map(|p| p.get_message().unwrap().to_owned())
        .collect::<Vec<_>>();
    let selected = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("How would you like to proceed")
        .items(&function_args_types)
        .default(0)
        .interact()
        .unwrap();
    match variants[selected] {
        FunctionArgsTypeDiscriminants::JsonArgs => Ok(FunctionArgsType::JsonArgs),
        FunctionArgsTypeDiscriminants::TextArgs => Ok(FunctionArgsType::TextArgs),
        FunctionArgsTypeDiscriminants::Base64Args => Ok(FunctionArgsType::Base64Args),
        FunctionArgsTypeDiscriminants::FileArgs => Ok(FunctionArgsType::FileArgs),
    }
}

pub fn function_args(
    args: String,
    function_args_type: FunctionArgsType,
) -> color_eyre::eyre::Result<Vec<u8>> {
    match function_args_type {
        super::call_function_args_type::FunctionArgsType::JsonArgs => {
            let data_json = serde_json::Value::from_str(&args).map_err(|err| {
                color_eyre::Report::msg(format!("Data not in JSON format! Error: {}", err))
            })?;
            Ok(data_json.to_string().into_bytes())
        }
        super::call_function_args_type::FunctionArgsType::TextArgs => Ok(args.into_bytes()),
        super::call_function_args_type::FunctionArgsType::Base64Args => {
            Ok(base64::decode(args.as_bytes())?)
        }
        super::call_function_args_type::FunctionArgsType::FileArgs => {
            let data_path = std::path::PathBuf::from(args);
            let data = std::fs::read(data_path).map_err(|err| {
                color_eyre::Report::msg(format!("Data file access not found! Error: {}", err))
            })?;
            Ok(data)
        }
    }
}
