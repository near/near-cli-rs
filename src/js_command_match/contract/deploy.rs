use near_gas::NearGas;

use crate::js_command_match::constants::{
    INIT_ARGS_ALIASES, INIT_DEPOSIT_ALIASES, INIT_FUNCTION_ALIASES, INIT_GAS_ALIASES,
    NETWORK_ID_ALIASES, WASM_FILE_ALIASES,
};

#[derive(Debug, Clone, clap::Parser)]
pub struct DeployArgs {
    account_id: String,
    #[clap(required_unless_present = "wasm_file" )]
    wasm_file_path: Option<String>,
    #[clap(long, aliases = WASM_FILE_ALIASES )]
    wasm_file: Option<String>,
    #[clap(long, aliases = INIT_FUNCTION_ALIASES, default_value=None)]
    init_function: Option<String>,
    #[clap(long, aliases = INIT_ARGS_ALIASES, default_value = "{}")]
    init_args: String,
    #[clap(long, aliases = INIT_GAS_ALIASES, default_value_t = 30_000_000_000_000)]
    init_gas: u64,
    #[clap(long, aliases = INIT_DEPOSIT_ALIASES, default_value = "0")]
    init_deposit: String,
    #[clap(long, aliases = NETWORK_ID_ALIASES, default_value=None)]
    network_id: Option<String>,
}

impl DeployArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);
        let mut command = vec!["contract".to_string(), "deploy".to_string()];

        command.push(self.account_id.to_owned());

        let wasm_file = self
            .wasm_file_path.to_owned()
            .or(self.wasm_file.to_owned()).unwrap();

        command.push("use-file".to_string());
        command.push(wasm_file.to_owned());

        if self.init_function.is_some() {
            command.push("with-init-call".to_string());
            command.push(self.init_function.to_owned().unwrap());
            command.push("json-args".to_string());
            command.push(self.init_args.to_owned());
            command.push("prepaid-gas".to_string());
            command.push(format!("{} tgas", NearGas::from_gas(self.init_gas).as_tgas()));
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

        let deploy_args = DeployArgs::parse_from(&["near", contract_account_id, wasm_file_path]);
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
    fn deploy_testnet_wasm_flag() {
        let contract_account_id = "bob.testnet";
        let wasm_file_path = "build/hello_near.wasm";

        let deploy_args = DeployArgs::parse_from(&["near", contract_account_id, "--wasmFile", wasm_file_path]);
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
                network_id,
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
        let args = format!(
            "{}{}{}{}",
            "{\"owner_id\":\"", contract_account_id, "}", ",\"total_supply\":\"1000000\"}"
        );

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
                    &args,
                ]);
                let result = DeployArgs::to_cli_args(&deploy_args, "testnet".to_string());
                assert_eq!(
                    result.join(" "),
                    format!(
                        "contract deploy {} use-file {} with-init-call {} json-args {} prepaid-gas 30000000000000 gas attached-deposit 0 NEAR network-config testnet sign-with-keychain send",
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
        let args = format!(
            "{}{}{}{}",
            "{\"owner_id\":\"", contract_account_id, "}", ",\"total_supply\":\"1000000\"}"
        );
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
                &init_gas.to_string(),
            ]);
            let result = DeployArgs::to_cli_args(&deploy_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!(
                    "contract deploy {} use-file {} with-init-call {} json-args {} prepaid-gas {} gas attached-deposit 0 NEAR network-config testnet sign-with-keychain send",
                    contract_account_id,
                    wasm_file_path,
                    init_function,
                    &args,
                    init_gas
                )
            );
        }
    }

    #[test]
    fn deploy_with_init_and_gas_and_deposit_testnet() {
        let contract_account_id = "bob.testnet";
        let wasm_file_path = "build/hello_near.wasm";
        let init_function = "new";
        let args = format!(
            "{}{}{}{}",
            "{\"owner_id\":\"", contract_account_id, "}", ",\"total_supply\":\"1000000\"}"
        );
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
                &init_deposit.to_string(),
            ]);
            let result = DeployArgs::to_cli_args(&deploy_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!(
                    "contract deploy {} use-file {} with-init-call {} json-args {} prepaid-gas {} gas attached-deposit {} NEAR network-config testnet sign-with-keychain send",
                    contract_account_id,
                    wasm_file_path,
                    init_function,
                    &args,
                    init_gas,
                    init_deposit
                )
            );
        }
    }
}
