use crate::js_command_match::constants::{
    DEFAULT_SEED_PHRASE_PATH, LEDGER_PATH_ALIASES, NETWORK_ID_ALIASES, SIGN_WITH_LEDGER_ALIASES,
};

#[derive(Debug, Clone, clap::Parser)]
#[clap(alias("delete"))]
pub struct DeleteAccountArgs {
    account_id: String,
    beneficiary_id: String,
    #[clap(long, aliases = SIGN_WITH_LEDGER_ALIASES, default_value_t = false)]
    sign_with_ledger: bool,
    #[clap(long, aliases = LEDGER_PATH_ALIASES, default_value = Some(DEFAULT_SEED_PHRASE_PATH))]
    ledger_path: Option<String>,
    #[clap(long, aliases = NETWORK_ID_ALIASES, default_value=None)]
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
            command.push(self.ledger_path.to_owned().unwrap_or_default());
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
    use super::*;
    use clap::Parser;

    #[test]
    fn delete_account_using_ledger_testnet() {
        let account_id = "bob.testnet";
        let beneficiary_id = "alice.testnet";

        for use_ledger_parameter_alias in SIGN_WITH_LEDGER_ALIASES {
            let delete_args = DeleteAccountArgs::parse_from(&[
                "near",
                account_id,
                beneficiary_id,
                &format!("--{use_ledger_parameter_alias}"),
                "--force",
            ]);
            let result = DeleteAccountArgs::to_cli_args(&delete_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!(
                    "account delete-account {account_id} beneficiary {beneficiary_id} network-config testnet sign-with-ledger --seed-phrase-hd-path {DEFAULT_SEED_PHRASE_PATH} send",
                )
            )
        }
    }

    #[test]
    fn delete_account_using_ledger_and_custom_path_testnet() {
        let account_id = "bob.testnet";
        let beneficiary_id = "alice.testnet";

        for use_ledger_alias in SIGN_WITH_LEDGER_ALIASES {
            let delete_args = DeleteAccountArgs::parse_from(&[
                "near",
                account_id,
                beneficiary_id,
                &format!("--{use_ledger_alias}"),
                "--ledgerPath",
                DEFAULT_SEED_PHRASE_PATH,
                "--force",
            ]);
            let result = DeleteAccountArgs::to_cli_args(&delete_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!(
                    "account delete-account {account_id} beneficiary {beneficiary_id} network-config testnet sign-with-ledger --seed-phrase-hd-path {DEFAULT_SEED_PHRASE_PATH} send",
                )
            )
        }
    }

    #[test]
    fn delete_account_using_ledger_mainnet() {
        let account_id = "bob.testnet";
        let beneficiary_id = "alice.testnet";
        let network_id = "mainnet";

        let delete_args = DeleteAccountArgs::parse_from(&[
            "near",
            account_id,
            beneficiary_id,
            "--signWithLedger",
            "--networkId",
            network_id,
            "--force",
        ]);
        let result = DeleteAccountArgs::to_cli_args(&delete_args, "testnet".to_string());
        assert_eq!(
                result.join(" "),
                format!(
                    "account delete-account {account_id} beneficiary {beneficiary_id} network-config {network_id} sign-with-ledger --seed-phrase-hd-path {DEFAULT_SEED_PHRASE_PATH} send",
                )
            )
    }

    #[test]
    fn delete_account_using_keychain_testnet() {
        let account_id = "bob.testnet";
        let beneficiary_id = "alice.testnet";
        let delete_args =
            DeleteAccountArgs::parse_from(&["near", account_id, beneficiary_id, "--force"]);
        let result = DeleteAccountArgs::to_cli_args(&delete_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            format!(
                "account delete-account {account_id} beneficiary {beneficiary_id} network-config testnet sign-with-keychain send",
            )
        );
    }
}
