use crate::js_command_match::constants::{
    DEFAULT_SEED_PHRASE_PATH, LEDGER_PATH_ALIASES, NETWORK_ID_ALIASES, SIGN_WITH_LEDGER_ALIASES,
};

#[derive(Debug, Clone, clap::Parser)]
pub struct DeleteAccountArgs {
    account_id: String,
    beneficiary_id: String,
    #[clap(long, aliases = SIGN_WITH_LEDGER_ALIASES, default_value_t = false)]
    sign_with_ledger: bool,
    #[clap(long, aliases = LEDGER_PATH_ALIASES, default_value = DEFAULT_SEED_PHRASE_PATH)]
    ledger_path: String,
    #[clap(long, aliases = NETWORK_ID_ALIASES)]
    network_id: Option<String>,
    #[clap(long, default_value_t = false)]
    force: bool,
}

impl DeleteAccountArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);

        let mut command = vec![
            "account".to_string(),
            "delete-account".to_string(),
            self.account_id.to_owned(),
            "beneficiary".to_string(),
            self.beneficiary_id.to_owned(),
        ];

        command.push("network-config".to_string());
        command.push(network_id);

        if self.sign_with_ledger {
            command.push("sign-with-ledger".to_string());
            command.push("--seed-phrase-hd-path".to_string());
            command.push(self.ledger_path.to_owned());
        } else {
            command.push("sign-with-keychain".to_string());
        }

        if self.force {
            command.push("send".to_string());
        }

        command
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::JsCmd;
    use super::*;
    use clap::Parser;

    #[test]
    fn delete_account() {
        for (input, expected_output) in [
            (
                "near delete bob.testnet alice.testnet --force".to_string(),
                "account delete-account bob.testnet beneficiary alice.testnet network-config testnet sign-with-keychain send".to_string()
            ),
            (
                "near delete-account bob.testnet alice.testnet --force".to_string(),
                "account delete-account bob.testnet beneficiary alice.testnet network-config testnet sign-with-keychain send".to_string()
            ),
            (
                format!("near delete-account bob.testnet alice.testnet --{} --force", SIGN_WITH_LEDGER_ALIASES[0]),
                "account delete-account bob.testnet beneficiary alice.testnet network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/1'\\''' send".to_string()
            ),
            (
                format!("near delete-account bob.testnet alice.testnet --{} --force", SIGN_WITH_LEDGER_ALIASES[1]),
                "account delete-account bob.testnet beneficiary alice.testnet network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/1'\\''' send".to_string()
            ),
            (
                format!("near delete-account bob.testnet alice.testnet --{} --force", SIGN_WITH_LEDGER_ALIASES[2]),
                "account delete-account bob.testnet beneficiary alice.testnet network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/1'\\''' send".to_string()
            ),
            (
                format!("near delete-account bob.testnet alice.testnet --{} --force", SIGN_WITH_LEDGER_ALIASES[3]),
                "account delete-account bob.testnet beneficiary alice.testnet network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/1'\\''' send".to_string()
            ),
            (
                format!("near delete-account bob.testnet alice.testnet --signWithLedger --{} \"44'/397'/0'/0'/2'\" --force", LEDGER_PATH_ALIASES[0]),
                "account delete-account bob.testnet beneficiary alice.testnet network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/2'\\''' send".to_string()
            ),
            (
                format!("near delete-account bob.testnet alice.testnet --signWithLedger --{} \"44'/397'/0'/0'/2'\" --force", LEDGER_PATH_ALIASES[1]),
                "account delete-account bob.testnet beneficiary alice.testnet network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/2'\\''' send".to_string()
            ),
            (
                format!("near delete-account bob.testnet alice.testnet --signWithLedger --{} mainnet --force", NETWORK_ID_ALIASES[0]),
                "account delete-account bob.testnet beneficiary alice.testnet network-config mainnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/1'\\''' send".to_string()
            ),
            (
                format!("near delete-account bob.testnet alice.testnet --signWithLedger --{} mainnet --force", NETWORK_ID_ALIASES[1]),
                "account delete-account bob.testnet beneficiary alice.testnet network-config mainnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/1'\\''' send".to_string()
            )
        ] {
            let input_cmd = shell_words::split(&input).expect("Input command must be a valid shell command");
            let JsCmd::DeleteAccount(delete_account_args) = JsCmd::parse_from(&input_cmd) else {
                panic!("DeleteAccount command was expected, but something else was parsed out from {input}");
            };
            assert_eq!(
                shell_words::join(DeleteAccountArgs::to_cli_args(&delete_account_args, "testnet".to_string())),
                expected_output
            );
        }
    }
}
