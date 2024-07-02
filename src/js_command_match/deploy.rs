use crate::js_command_match::constants::{
    WASM_FILE_ALIASES,
    INIT_FUNCTION_ALIASES,
    INIT_ARGS_ALIASES,
    INIT_GAS_ALIASES,
    INIT_DEPOSIT_ALIASES,
    NETWORK_ID_ALIASES,
};

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `deploy` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct DeployArgs {
    contract_account_id: String,
    // #[clap(long, aliases = ["account_id", "accountId", "contract_name", "contractName"])]
    // account_id: Option<String>,
    wasm_file_path: Option<String>,
    #[clap(long, aliases = WASM_FILE_ALIASES)]
    wasm_file: Option<String>,
    #[clap(long, aliases = INIT_FUNCTION_ALIASES)]
    init_function: Option<String>,
    #[clap(long, aliases = INIT_ARGS_ALIASES)]
    init_args: Option<String>,
    #[clap(long, aliases = INIT_GAS_ALIASES, default_value_t = 30_000_000_000_000)]
    init_gas: u64,
    #[clap(long, aliases = INIT_DEPOSIT_ALIASES, default_value = "0")]
    init_deposit: String,
    #[clap(long, aliases = NETWORK_ID_ALIASES, default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl DeployArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);
        let mut command = vec![
            "contract".to_string(),
            "deploy".to_string(),
        ];

        // let contract_account_id = if let Some(account_id) = &self.contract_account_id {
        //     account_id
        // } else if let Some(account_id) = &self.account_id {
        //     account_id
        // } else {
        //     return command;
        // };

        command.push(self.contract_account_id.to_owned());

        let wasm_file = if let Some(file_path) = &self.wasm_file_path {
            file_path
        } else if let Some(wasm_file) = &self.wasm_file {
            wasm_file
        } else {
            return command;
        };

        command.push("use-file".to_string());
        command.push(wasm_file.to_owned());
        
        if self.init_function.is_some() {
            command.push("with-init-call".to_string());
            command.push(self.init_function.as_deref().unwrap_or("new").to_owned());
            command.push("json-args".to_string());
            command.push(self.init_args.as_deref().unwrap_or("{}").to_owned());
            command.push("prepaid-gas".to_string());
            command.push(format!("{} TeraGas", self.init_gas / 1_000_000_000_000));
            command.push("attached-deposit".to_string());
            command.push(format!("{} NEAR", self.init_deposit));
        } else {
            command.push("without-init-call".to_string());
        }

        command.push("network-config".to_string());
        command.push(network_id);
        command.push("sign-with-keychain".to_string());
        command.push("send".to_owned());

        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
 
    #[test]
    fn deploy_testnet() {
        let contract_account_id = "bob.testnet";
        let wasm_file_path = "build/hello_near.wasm";

        let deploy_args = DeployArgs::parse_from(&[
            "near",
            contract_account_id,
            wasm_file_path
        ]);
        let result = DeployArgs::to_cli_args(&deploy_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            format!(
                "contract deploy {} use-file {} without-init-call network-config testnet sign-with-keychain send",
                contract_account_id,
                wasm_file_path,
            )
        );
    }

    #[test]
    fn deploy_mainnet() {
        let contract_account_id = "bob.testnet";
        let wasm_file_path = "build/hello_near.wasm";
        let network_id: &str = "mainnet";

        for i in 0..NETWORK_ID_ALIASES.len() {
            let network_id_parameter_alias = &format!("--{}", &NETWORK_ID_ALIASES[i]);
            let deploy_args = DeployArgs::parse_from(&[
                "near",
                contract_account_id,
                wasm_file_path,
                network_id_parameter_alias,
                network_id
            ]);
            let result = DeployArgs::to_cli_args(&deploy_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!(
                    "contract deploy {} use-file {} without-init-call network-config {} sign-with-keychain send",
                    contract_account_id,
                    wasm_file_path,
                    network_id
                )
            );
        }
    }

    #[test]
    fn deploy_with_init_testnet() {
        let contract_account_id = "bob.testnet";
        let wasm_file_path = "build/hello_near.wasm";
        let init_function = "new";
        let args = format!("{}{}{}{}", "{\"owner_id\":\"", contract_account_id, "}", ",\"total_supply\":\"1000000\"}");

        for i in 0..INIT_FUNCTION_ALIASES.len() {
            let init_function_parameter_alias = &format!("--{}", &INIT_FUNCTION_ALIASES[i]);

            for j in 0..INIT_ARGS_ALIASES.len() {
                let init_args_parameter_alias = &format!("--{}", &INIT_ARGS_ALIASES[j]);
                let deploy_args = DeployArgs::parse_from(&[
                    "near",
                    contract_account_id,
                    wasm_file_path,
                    init_function_parameter_alias,
                    init_function,
                    init_args_parameter_alias,
                    &args
                ]);
                let result = DeployArgs::to_cli_args(&deploy_args, "testnet".to_string());
                assert_eq!(
                    result.join(" "),
                    format!(
                        "contract deploy {} use-file {} with-init-call {} json-args {} prepaid-gas 30 TeraGas attached-deposit 0 NEAR network-config testnet sign-with-keychain send",
                        contract_account_id,
                        wasm_file_path,
                        init_function,
                        &args,
                    )
                );
            }
        }
    }
    
    #[test]
    fn deploy_with_init_and_gas_testnet() {
        let contract_account_id = "bob.testnet";
        let wasm_file_path = "build/hello_near.wasm";
        let init_function = "new";
        let args = format!("{}{}{}{}", "{\"owner_id\":\"", contract_account_id, "}", ",\"total_supply\":\"1000000\"}");
        let init_gas: i64 = 60000000000000;

        for i in 0..INIT_GAS_ALIASES.len() {
            let init_gas_parameter_alias = &format!("--{}", &INIT_GAS_ALIASES[i]);
            let init_function_parameter_alias = &format!("--{}", &INIT_FUNCTION_ALIASES[0]);
            let init_args_parameter_alias = &format!("--{}", &INIT_ARGS_ALIASES[0]);

            let deploy_args = DeployArgs::parse_from(&[
                "near",
                contract_account_id,
                wasm_file_path,
                init_function_parameter_alias,
                init_function,
                init_args_parameter_alias,
                &args,
                init_gas_parameter_alias,
                &init_gas.to_string()
            ]);
            let result = DeployArgs::to_cli_args(&deploy_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!(
                    "contract deploy {} use-file {} with-init-call {} json-args {} prepaid-gas {} TeraGas attached-deposit 0 NEAR network-config testnet sign-with-keychain send",
                    contract_account_id,
                    wasm_file_path,
                    init_function,
                    &args,
                    init_gas / 1_000_000_000_000
                )
            );
        }
    }

    #[test]
    fn deploy_with_init_and_gas_and_deposit_testnet() {
        let contract_account_id = "bob.testnet";
        let wasm_file_path = "build/hello_near.wasm";
        let init_function = "new";
        let args = format!("{}{}{}{}", "{\"owner_id\":\"", contract_account_id, "}", ",\"total_supply\":\"1000000\"}");
        let init_gas: i64 = 60000000000000;
        let init_deposit = 1;

        for i in 0..INIT_DEPOSIT_ALIASES.len() {
          let init_deposit_parameter_alias = &format!("--{}", &INIT_DEPOSIT_ALIASES[i]);
          let init_function_parameter_alias = &format!("--{}", &INIT_FUNCTION_ALIASES[0]);
          let init_args_parameter_alias = &format!("--{}", &INIT_ARGS_ALIASES[0]);
          let init_gas_parameter_alias = &format!("--{}", &INIT_GAS_ALIASES[0]);

            let deploy_args = DeployArgs::parse_from(&[
                "near",
                contract_account_id,
                wasm_file_path,
                init_function_parameter_alias,
                init_function,
                init_args_parameter_alias,
                &args,
                init_gas_parameter_alias,
                &init_gas.to_string(),
                init_deposit_parameter_alias,
                &init_deposit.to_string()
            ]);
            let result = DeployArgs::to_cli_args(&deploy_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!(
                    "contract deploy {} use-file {} with-init-call {} json-args {} prepaid-gas {} TeraGas attached-deposit {} NEAR network-config testnet sign-with-keychain send",
                    contract_account_id,
                    wasm_file_path,
                    init_function,
                    &args,
                    init_gas / 1_000_000_000_000,
                    init_deposit
                )
            );
        }
    }
}