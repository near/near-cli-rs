use crate::js_command_match::parameter_aliases::{
  CONTRACT_ID_ALIASES,
  METHOD_NAMES_ALIASES,
  NETWORK_ID_ALIASES
};

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `add-key` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct AddKeyArgs {
    account_id: String,
    access_key: String,
    #[clap(long, aliases = CONTRACT_ID_ALIASES, default_value = None)]
    contract_id: Option<String>,
    #[clap(long, aliases = METHOD_NAMES_ALIASES, requires = "contract_id", value_delimiter = ',', num_args = 1..)]
    method_names: Vec<String>,
    #[clap(long, default_value = "0")]
    allowance: String,
    #[clap(long, aliases = NETWORK_ID_ALIASES, default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl AddKeyArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);

        let mut command = vec![
            "account".to_string(),
            "add-key".to_string(),
            self.account_id.to_owned(),
        ];

        if let Some(contract_id) = self.contract_id.as_deref() {
            command.push("grant-function-call-access".to_string());
            command.push("--allowance".to_string());
            command.push(format!("{} NEAR", self.allowance));
            command.push("--receiver-account-id".to_string());
            command.push(contract_id.to_owned());
            command.push("--method-names".to_string());
            command.push(self.method_names.join(","));
        } else {
            command.push("grant-full-access".to_string());
        }
          
        command.push("use-manually-provided-public-key".to_string());
        command.push(self.access_key.to_owned());
        command.push("network-config".to_string());
        command.push(network_id);
        command.push("sign-with-keychain".to_string());
        command.push("send".to_string());

        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn add_function_call_access_key_for_all_methods_testnet() {
        let account_id = "bob.testnet";
        let access_key = "ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq";
        let contract_id = "example.testnet";

        for i in 0..CONTRACT_ID_ALIASES.len() {
            let contract_id_parameter_alias = &format!("--{}", &CONTRACT_ID_ALIASES[i]);
            let add_key_args = AddKeyArgs::parse_from(&[
                "near",
                account_id,
                access_key,
                contract_id_parameter_alias,
                contract_id,
            ]);
            let result = AddKeyArgs::to_cli_args(&add_key_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!("account add-key {} grant-function-call-access --allowance 0 NEAR --receiver-account-id {} --method-names  use-manually-provided-public-key {} network-config testnet sign-with-keychain send", account_id, contract_id, access_key)
            )
        }
    }

    #[test]
    fn add_function_call_access_key_for_some_methods_testnet() {
        let account_id = "bob.testnet";
        let access_key = "ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq";
        let contract_id = "example.testnet";
        let method_names = "get,set";

        for i in 0..METHOD_NAMES_ALIASES.len() {
            let method_names_parameter_alias = &format!("--{}", &METHOD_NAMES_ALIASES[i]);
            let contract_id_parameter_alias = &format!("--{}", &CONTRACT_ID_ALIASES[0]);
            let add_key_args = AddKeyArgs::parse_from(&[
                "near",
                account_id,
                access_key,
                contract_id_parameter_alias,
                contract_id,
                method_names_parameter_alias,
                method_names
            ]);
            let result = AddKeyArgs::to_cli_args(&add_key_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!("account add-key {} grant-function-call-access --allowance 0 NEAR --receiver-account-id {} --method-names {} use-manually-provided-public-key {} network-config testnet sign-with-keychain send", account_id, contract_id, method_names, access_key)
            )
        }
    }

    #[test]
    fn add_full_access_key_testnet() {
        let account_id = "bob.testnet";
        let access_key = "ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq";

        for i in 0..NETWORK_ID_ALIASES.len() {
            let network_id_parameter_alias = &format!("--{}", &NETWORK_ID_ALIASES[i]);
            let add_key_args = AddKeyArgs::parse_from(&[
                "near",
                account_id,
                access_key,
                network_id_parameter_alias,
                "testnet"
            ]);
            let result = AddKeyArgs::to_cli_args(&add_key_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!("account add-key {} grant-full-access use-manually-provided-public-key {} network-config testnet sign-with-keychain send", account_id, access_key)
            )
        }
    }

    #[test]
    fn add_full_access_key_mainnet() {
        let account_id = "bob.testnet";
        let access_key = "ed25519:DReZmNmnGhpsYcCFFeYgPsJ9YCm9xH16GGujCPe3KQEq";
        let network_id = "mainnet";

        for i in 0..NETWORK_ID_ALIASES.len() {
            let network_id_parameter_alias = &format!("--{}", &NETWORK_ID_ALIASES[i]);
            let add_key_args = AddKeyArgs::parse_from(&[
                "near",
                account_id,
                access_key,
                network_id_parameter_alias,
                network_id
            ]);
            let result = AddKeyArgs::to_cli_args(&add_key_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!("account add-key {} grant-full-access use-manually-provided-public-key {} network-config {} sign-with-keychain send", account_id, access_key, network_id)
            )
        }
    }
}