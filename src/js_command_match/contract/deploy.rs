use near_gas::NearGas;

use crate::js_command_match::constants::{
    INIT_ARGS_ALIASES, INIT_DEPOSIT_ALIASES, INIT_FUNCTION_ALIASES, INIT_GAS_ALIASES,
    NETWORK_ID_ALIASES, WASM_FILE_ALIASES,
};

#[derive(Debug, Clone, clap::Parser)]
pub struct DeployArgs {
    account_id: String,
    #[clap(required_unless_present = "wasm_file")]
    wasm_file_path: Option<String>,
    #[clap(long, aliases = WASM_FILE_ALIASES )]
    wasm_file: Option<String>,
    #[clap(long, aliases = INIT_FUNCTION_ALIASES)]
    init_function: Option<String>,
    #[clap(long, aliases = INIT_ARGS_ALIASES, default_value = "{}")]
    init_args: String,
    #[clap(long, aliases = INIT_GAS_ALIASES, default_value_t = 30_000_000_000_000)]
    init_gas: u64,
    #[clap(long, aliases = INIT_DEPOSIT_ALIASES, default_value = "0")]
    init_deposit: String,
    #[clap(long, aliases = NETWORK_ID_ALIASES)]
    network_id: Option<String>,
}

impl DeployArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);
        let mut command = vec!["contract".to_string(), "deploy".to_string()];

        command.push(self.account_id.to_owned());

        let wasm_file = self
            .wasm_file_path
            .to_owned()
            .or(self.wasm_file.to_owned())
            .unwrap();

        command.push("use-file".to_string());
        command.push(wasm_file.to_owned());

        if let Some(init_function) = &self.init_function {
            command.push("with-init-call".to_string());
            command.push(init_function.to_string());
            command.push("json-args".to_string());
            command.push(self.init_args.to_owned());
            command.push("prepaid-gas".to_string());
            command.push(format!(
                "{} Tgas",
                NearGas::from_gas(self.init_gas).as_tgas()
            ));
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
    use super::super::super::JsCmd;
    use super::*;
    use clap::Parser;

    #[test]
    fn deploy() {
        let args = "{\"owner_id\":\"contract.testnet\",\"total_supply\":\"1000000\"}";

        for (input, expected_output) in [
            (
                "near deploy contract.testnet build/hello_near.wasm".to_string(),
                "contract deploy contract.testnet use-file build/hello_near.wasm without-init-call network-config testnet sign-with-keychain send".to_string(),
            ),
            (
                format!("near deploy contract.testnet --{} build/hello_near.wasm", WASM_FILE_ALIASES[0]),
                "contract deploy contract.testnet use-file build/hello_near.wasm without-init-call network-config testnet sign-with-keychain send".to_string(),
            ),
            (
                format!("near deploy contract.testnet --{} build/hello_near.wasm", WASM_FILE_ALIASES[1]),
                "contract deploy contract.testnet use-file build/hello_near.wasm without-init-call network-config testnet sign-with-keychain send".to_string(),
            ),
            (
                format!("near deploy contract.testnet build/hello_near.wasm --{} new --initArgs '{args}'", INIT_FUNCTION_ALIASES[0]),
                format!("contract deploy contract.testnet use-file build/hello_near.wasm with-init-call new json-args '{args}' prepaid-gas '30 Tgas' attached-deposit '0 NEAR' network-config testnet sign-with-keychain send")
            ),
            (
                format!("near deploy contract.testnet build/hello_near.wasm --{} new --initArgs '{args}'", INIT_FUNCTION_ALIASES[1]),
                format!("contract deploy contract.testnet use-file build/hello_near.wasm with-init-call new json-args '{args}' prepaid-gas '30 Tgas' attached-deposit '0 NEAR' network-config testnet sign-with-keychain send")
            ),
            (
                format!("near deploy contract.testnet build/hello_near.wasm --initFunction new --{} '{args}'", INIT_ARGS_ALIASES[0]),
                format!("contract deploy contract.testnet use-file build/hello_near.wasm with-init-call new json-args '{args}' prepaid-gas '30 Tgas' attached-deposit '0 NEAR' network-config testnet sign-with-keychain send")
            ),
            (
                format!("near deploy contract.testnet build/hello_near.wasm --initFunction new --{} '{args}'", INIT_ARGS_ALIASES[1]),
                format!("contract deploy contract.testnet use-file build/hello_near.wasm with-init-call new json-args '{args}' prepaid-gas '30 Tgas' attached-deposit '0 NEAR' network-config testnet sign-with-keychain send")
            ),
            (
                format!("near deploy contract.testnet build/hello_near.wasm --initFunction new --initArgs '{args}' --{} 60000000000000", INIT_GAS_ALIASES[0]),
                format!("contract deploy contract.testnet use-file build/hello_near.wasm with-init-call new json-args '{args}' prepaid-gas '60 Tgas' attached-deposit '0 NEAR' network-config testnet sign-with-keychain send")
            ),
            (
                format!("near deploy contract.testnet build/hello_near.wasm --initFunction new --initArgs '{args}' --{} 60000000000000", INIT_GAS_ALIASES[1]),
                format!("contract deploy contract.testnet use-file build/hello_near.wasm with-init-call new json-args '{args}' prepaid-gas '60 Tgas' attached-deposit '0 NEAR' network-config testnet sign-with-keychain send")
            ),
            (
                format!("near deploy contract.testnet build/hello_near.wasm --initFunction new --initArgs '{args}' --initGas 60000000000000 --{} 1", INIT_DEPOSIT_ALIASES[0]),
                format!("contract deploy contract.testnet use-file build/hello_near.wasm with-init-call new json-args '{args}' prepaid-gas '60 Tgas' attached-deposit '1 NEAR' network-config testnet sign-with-keychain send")
            ),
            (
                format!("near deploy contract.testnet build/hello_near.wasm --initFunction new --initArgs '{args}' --initGas 60000000000000 --{} 1", INIT_DEPOSIT_ALIASES[1]),
                format!("contract deploy contract.testnet use-file build/hello_near.wasm with-init-call new json-args '{args}' prepaid-gas '60 Tgas' attached-deposit '1 NEAR' network-config testnet sign-with-keychain send")
            ),
            (
                format!("near deploy contract.testnet build/hello_near.wasm --{} testnet", NETWORK_ID_ALIASES[0]),
                "contract deploy contract.testnet use-file build/hello_near.wasm without-init-call network-config testnet sign-with-keychain send".to_string(),
            ),
            (
                format!("near deploy contract.testnet build/hello_near.wasm --{} mainnet", NETWORK_ID_ALIASES[1]),
                "contract deploy contract.testnet use-file build/hello_near.wasm without-init-call network-config mainnet sign-with-keychain send".to_string(),
            ),
        ] {
            let input_cmd = shell_words::split(&input).expect("Input command must be a valid shell command");
            let JsCmd::Deploy(deploy_args) = JsCmd::parse_from(&input_cmd) else {
                panic!("Deploy command was expected, but something else was parsed out from {input}");
            };
            assert_eq!(
                shell_words::join(DeployArgs::to_cli_args(&deploy_args, "testnet".to_string())),
                expected_output
            );
        }
    }
}
