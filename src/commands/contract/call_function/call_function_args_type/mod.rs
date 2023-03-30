use std::str::FromStr;

use color_eyre::eyre::Context;
use inquire::Select;
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

#[derive(Debug, EnumDiscriminants, Clone, clap::ValueEnum)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How do you want to pass the function call arguments?
pub enum FunctionArgsType {
    #[strum_discriminants(strum(
        message = "json-args    - Valid JSON arguments (e.g. {\"token_id\": \"42\"})"
    ))]
    /// Valid JSON arguments (e.g. {"token_id": "42"})
    Json,
    #[strum_discriminants(strum(message = "text-args    - Arbitrary text arguments"))]
    /// Arbitrary text arguments
    Text,
    #[strum_discriminants(strum(message = "base64-args  - Base64-encoded string (e.g. e30=)"))]
    /// Base64-encoded string (e.g. e30=)
    Base64,
    #[strum_discriminants(strum(
        message = "file-args    - Read from file (e.g. reusable JSON or binary data)"
    ))]
    /// Read from file (e.g. reusable JSON or binary data)
    File,
}

impl interactive_clap::ToCli for FunctionArgsType {
    type CliVariant = FunctionArgsType;
}

impl std::str::FromStr for FunctionArgsType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json-args" => Ok(Self::Json),
            "text-args" => Ok(Self::Text),
            "base64-args" => Ok(Self::Base64),
            "file-args" => Ok(Self::File),
            _ => Err("FunctionArgsType: incorrect value entered".to_string()),
        }
    }
}

impl std::fmt::Display for FunctionArgsType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json-args"),
            Self::Text => write!(f, "text-args"),
            Self::Base64 => write!(f, "base64-args"),
            Self::File => write!(f, "file-args"),
        }
    }
}

impl std::fmt::Display for FunctionArgsTypeDiscriminants {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json-args"),
            Self::Text => write!(f, "text-args"),
            Self::Base64 => write!(f, "base64-args"),
            Self::File => write!(f, "file-args"),
        }
    }
}

pub fn input_function_args_type() -> color_eyre::eyre::Result<Option<FunctionArgsType>> {
    let variants = FunctionArgsTypeDiscriminants::iter().collect::<Vec<_>>();
    let selected = Select::new(" How would you like to proceed", variants).prompt()?;
    match selected {
        FunctionArgsTypeDiscriminants::Json => Ok(Some(FunctionArgsType::Json)),
        FunctionArgsTypeDiscriminants::Text => Ok(Some(FunctionArgsType::Text)),
        FunctionArgsTypeDiscriminants::Base64 => Ok(Some(FunctionArgsType::Base64)),
        FunctionArgsTypeDiscriminants::File => Ok(Some(FunctionArgsType::File)),
    }
}

pub fn function_args(
    args: String,
    function_args_type: FunctionArgsType,
) -> color_eyre::eyre::Result<Vec<u8>> {
    match function_args_type {
        super::call_function_args_type::FunctionArgsType::Json => {
            let data_json =
                serde_json::Value::from_str(&args).wrap_err("Data not in JSON format!")?;
            Ok(data_json.to_string().into_bytes())
        }
        super::call_function_args_type::FunctionArgsType::Text => Ok(args.into_bytes()),
        super::call_function_args_type::FunctionArgsType::Base64 => {
            Ok(base64::decode(args.as_bytes())?)
        }
        super::call_function_args_type::FunctionArgsType::File => {
            let data_path = std::path::PathBuf::from(args);
            let data = std::fs::read(&data_path)
                .wrap_err_with(|| format!("Access to data file <{:?}> not found!", &data_path))?;
            Ok(data)
        }
    }
}
