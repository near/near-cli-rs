use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod json_args;
mod text_args;

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::UpdateAccountProfileContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How do you want to pass profile arguments?
pub enum ProfileArgsType {
    #[strum_discriminants(strum(
        message = "json-args    - Valid JSON arguments (e.g. {\"token_id\": \"42\"})"
    ))]
    /// Valid JSON arguments (e.g. {"token_id": "42"})
    JsonArgs(self::json_args::JsonArgs),
    #[strum_discriminants(strum(message = "text-args    - Arbitrary text arguments"))]
    /// Arbitrary text arguments
    TextArgs(self::text_args::TextArgs),
    #[strum_discriminants(strum(message = "base64-args  - Base64-encoded string (e.g. e30=)"))]
    /// Base64-encoded string (e.g. e30=)
    Base64Args,
    #[strum_discriminants(strum(
        message = "file-args    - Read from file (e.g. reusable JSON or binary data)"
    ))]
    /// Read from file (e.g. reusable JSON or binary data)
    FileArgs,
    #[strum_discriminants(strum(message = "manually     - Interactive input of arguments"))]
    /// Interactive input of arguments
    Manually,
}

#[derive(Clone)]
pub struct ArgsContext {
    pub global_context: crate::GlobalContext,
    pub get_contract_account_id: super::super::super::storage_management::GetContractAccountId,
    pub account_id: near_primitives::types::AccountId,
    pub data: String,
}

// impl interactive_clap::ToCli for ProfileArgsType {
//     type CliVariant = ProfileArgsType;
// }

// impl std::str::FromStr for ProfileArgsType {
//     type Err = String;
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             "json-args" => Ok(Self::JsonArgs),
//             "text-args" => Ok(Self::TextArgs),
//             "base64-args" => Ok(Self::Base64Args),
//             "file-args" => Ok(Self::FileArgs),
//             _ => Err("FunctionArgsType: incorrect value entered".to_string()),
//         }
//     }
// }

// impl std::fmt::Display for ProfileArgsType {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         match self {
//             Self::JsonArgs => write!(f, "json-args"),
//             Self::TextArgs => write!(f, "text-args"),
//             Self::Base64Args => write!(f, "base64-args"),
//             Self::FileArgs => write!(f, "file-args"),
//         }
//     }
// }

// impl std::fmt::Display for FunctionArgsTypeDiscriminants {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         match self {
//             Self::JsonArgs => write!(f, "json-args"),
//             Self::TextArgs => write!(f, "text-args"),
//             Self::Base64Args => write!(f, "base64-args"),
//             Self::FileArgs => write!(f, "file-args"),
//         }
//     }
// }

// pub fn input_function_args_type() -> color_eyre::eyre::Result<Option<ProfileArgsType>> {
//     let variants = FunctionArgsTypeDiscriminants::iter().collect::<Vec<_>>();
//     let selected = Select::new("How would you like to proceed?", variants).prompt()?;
//     match selected {
//         FunctionArgsTypeDiscriminants::JsonArgs => Ok(Some(ProfileArgsType::JsonArgs)),
//         FunctionArgsTypeDiscriminants::TextArgs => Ok(Some(ProfileArgsType::TextArgs)),
//         FunctionArgsTypeDiscriminants::Base64Args => Ok(Some(ProfileArgsType::Base64Args)),
//         FunctionArgsTypeDiscriminants::FileArgs => Ok(Some(ProfileArgsType::FileArgs)),
//     }
// }

// pub fn function_args(
//     args: String,
//     function_args_type: ProfileArgsType,
// ) -> color_eyre::eyre::Result<Vec<u8>> {
//     match function_args_type {
//         super::profile_args_type_c::ProfileArgsType::JsonArgs => {
//             let data_json =
//                 serde_json::Value::from_str(&args).wrap_err("Data not in JSON format!")?;
//             Ok(data_json.to_string().into_bytes())
//         }
//         super::profile_args_type_c::ProfileArgsType::TextArgs => Ok(args.into_bytes()),
//         super::profile_args_type_c::ProfileArgsType::Base64Args => {
//             Ok(near_primitives::serialize::from_base64(&args)
//                 .map_err(|_| color_eyre::eyre::eyre!("Data cannot be decoded with base64"))?)
//         }
//         super::profile_args_type_c::ProfileArgsType::FileArgs => {
//             let data_path = std::path::PathBuf::from(args);
//             let data = std::fs::read(&data_path)
//                 .wrap_err_with(|| format!("Access to data file <{:?}> not found!", &data_path))?;
//             Ok(data)
//         }
//     }
// }
