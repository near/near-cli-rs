use crate::js_command_match::constants::{
    DEFAULT_SEED_PHRASE_PATH, INITIAL_BALANCE_ALIASES, LEDGER_PATH_ALIASES, NETWORK_ID_ALIASES,
    PK_LEDGER_PATH_ALIASES, PUBLIC_KEY_ALIASES, SEED_PHRASE_ALIASES, SIGN_WITH_LEDGER_ALIASES,
    USE_ACCOUNT_ALIASES, USE_FAUCET_ALIASES, USE_LEDGER_PK_ALIASES,
};

#[derive(Debug, Clone, clap::Parser)]
#[clap(alias("create"))]
pub struct CreateAccountArgs {
    new_account_id: String,
    #[clap(long, aliases = USE_FAUCET_ALIASES, default_value_t = false)]
    use_faucet: bool,
    #[clap(long, aliases = USE_ACCOUNT_ALIASES, conflicts_with = "use_faucet")]
    use_account: Option<String>,
    #[clap(long, aliases = INITIAL_BALANCE_ALIASES, default_value = "1")]
    initial_balance: String,
    #[clap(long, aliases = PUBLIC_KEY_ALIASES)]
    public_key: Option<String>,
    #[clap(long, aliases = SEED_PHRASE_ALIASES, conflicts_with = "public_key")]
    seed_phrase: Option<String>,
    #[clap(long, aliases = SIGN_WITH_LEDGER_ALIASES, default_value_t = false, conflicts_with="use_faucet")]
    sign_with_ledger: bool,
    #[clap(long, aliases = LEDGER_PATH_ALIASES, default_value = DEFAULT_SEED_PHRASE_PATH)]
    ledger_path: String,
    #[clap(long, aliases = USE_LEDGER_PK_ALIASES, default_value_t = false, conflicts_with = "public_key")]
    use_ledger_pk: bool,
    #[clap(long, aliases = PK_LEDGER_PATH_ALIASES, default_value = DEFAULT_SEED_PHRASE_PATH)]
    pk_ledger_path: String,
    #[clap(long, aliases = NETWORK_ID_ALIASES)]
    network_id: Option<String>,
}

impl CreateAccountArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);

        let mut command = vec!["account".to_string(), "create-account".to_string()];

        if self.use_faucet {
            command.push("sponsor-by-faucet-service".to_string());
            command.push(self.new_account_id.to_owned());
        } else {
            command.push("fund-myself".to_string());
            command.push(self.new_account_id.to_owned());
            command.push(format!("{} NEAR", self.initial_balance));
        }

        if self.use_ledger_pk {
            command.push("use-ledger".to_owned());

            // add after issue with ledger key is resolved
            // command.push(format!("--seed-phrase-hd-path {}", self.pk_ledger_path.clone().unwrap()));
        } else if let Some(seed_phrase) = &self.seed_phrase {
            command.push("use-manually-provided-seed-phrase".to_string());
            command.push(seed_phrase.to_string());
        } else if let Some(public_key) = &self.public_key {
            command.push("use-manually-provided-public-key".to_string());
            command.push(public_key.to_string());
        } else {
            command.push("autogenerate-new-keypair".to_string());
            command.push("save-to-keychain".to_string());
        }

        if !self.use_faucet {
            command.push("sign-as".to_string());
            command.push(
                self.use_account
                    .to_owned()
                    .expect("Valid master account must be provided"),
            );
        };

        command.push("network-config".to_string());
        command.push(network_id);

        if self.use_faucet {
            command.push("create".to_string());
        } else {
            if self.sign_with_ledger {
                command.push("sign-with-ledger".to_string());
                command.push("--seed-phrase-hd-path".to_string());
                command.push(self.ledger_path.to_owned());
            } else {
                command.push("sign-with-keychain".to_string());
            }
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
    fn create_account_v1() {
        for (input, expected_output) in [
            (
                format!("near create bob.testnet --{}", USE_FAUCET_ALIASES[0]),
                "account create-account sponsor-by-faucet-service bob.testnet autogenerate-new-keypair save-to-keychain network-config testnet create"
            ),
            (
                format!("near create bob.testnet --{}", USE_FAUCET_ALIASES[1]),
                "account create-account sponsor-by-faucet-service bob.testnet autogenerate-new-keypair save-to-keychain network-config testnet create"
            ),
            (
                format!("near create bob.testnet --{} alice.testnet", USE_ACCOUNT_ALIASES[0]),
                "account create-account fund-myself bob.testnet '1 NEAR' autogenerate-new-keypair save-to-keychain sign-as alice.testnet network-config testnet sign-with-keychain send"
            ),
            (
                format!("near create bob.testnet --useAccount alice.testnet --{} 0.1", INITIAL_BALANCE_ALIASES[0]),
                "account create-account fund-myself bob.testnet '0.1 NEAR' autogenerate-new-keypair save-to-keychain sign-as alice.testnet network-config testnet sign-with-keychain send"
            ),
            (
                format!("near create bob.testnet --{} 'crisp clump stay mean dynamic become fashion mail bike disorder chronic sight' --useFaucet", SEED_PHRASE_ALIASES[0]),
                "account create-account sponsor-by-faucet-service bob.testnet use-manually-provided-seed-phrase 'crisp clump stay mean dynamic become fashion mail bike disorder chronic sight' network-config testnet create"
            ),
            (
                format!("near create bob.testnet --useAccount alice.testnet --{} 78MziB9aTNsu19MHHVrfWy762S5mAqXgCB6Vgvrv9uGV --initialBalance 0.1", PUBLIC_KEY_ALIASES[0]),
                "account create-account fund-myself bob.testnet '0.1 NEAR' use-manually-provided-public-key 78MziB9aTNsu19MHHVrfWy762S5mAqXgCB6Vgvrv9uGV sign-as alice.testnet network-config testnet sign-with-keychain send"
            ),
            (
                format!("near create bob.testnet --{} --useFaucet", USE_LEDGER_PK_ALIASES[0]),
                "account create-account sponsor-by-faucet-service bob.testnet use-ledger network-config testnet create"
            ),
            (
                "near create bob.testnet --useAccount alice.testnet --signWithLedger --networkId testnet".to_string(),
                "account create-account fund-myself bob.testnet '1 NEAR' autogenerate-new-keypair save-to-keychain sign-as alice.testnet network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/1'\\''' send"
            ),
            (
                "near create bob.near --useAccount alice.near --signWithLedger --ledgerPath \"44'/397'/0'/0'/2'\" --networkId mainnet".to_string(),
                "account create-account fund-myself bob.near '1 NEAR' autogenerate-new-keypair save-to-keychain sign-as alice.near network-config mainnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/2'\\''' send"
            )
        ] {
            let input_cmd = shell_words::split(&input).expect("Input command must be a valid shell command");
            let JsCmd::CreateAccount(create_account_args) = JsCmd::parse_from(&input_cmd) else {
                panic!("CreateAccount command was expected, but something else was parsed out from {input}");
            };
            assert_eq!(
                shell_words::join(CreateAccountArgs::to_cli_args(&create_account_args, "testnet".to_string())),
                expected_output
            );
        }
    }

    #[test]
    fn create_account_v2() {
        for (input, expected_output) in [
            (
                "near create bob.testnet --useAccount alice.testnet --signWithLedger --networkId testnet".to_string(),
                "account create-account fund-myself bob.testnet '1 NEAR' autogenerate-new-keypair save-to-keychain sign-as alice.testnet network-config testnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/1'\\''' send"
            ),
            (
                "near create bob.near --useAccount alice.near --signWithLedger --ledgerPath \"44'/397'/0'/0'/2'\" --networkId mainnet".to_string(),
                "account create-account fund-myself bob.near '1 NEAR' autogenerate-new-keypair save-to-keychain sign-as alice.near network-config mainnet sign-with-ledger --seed-phrase-hd-path '44'\\''/397'\\''/0'\\''/0'\\''/2'\\''' send"
            )
        ].into_iter().chain(
            USE_FAUCET_ALIASES.iter().map(|use_faucet_alias| (
                format!("near create bob.testnet --{use_faucet_alias}"),
                "account create-account sponsor-by-faucet-service bob.testnet autogenerate-new-keypair save-to-keychain network-config testnet create"
            ))
        ).chain(
            USE_ACCOUNT_ALIASES.iter().map(|use_account_alias| (
                format!("near create bob.testnet --{use_account_alias} alice.testnet"),
                "account create-account fund-myself bob.testnet '1 NEAR' autogenerate-new-keypair save-to-keychain sign-as alice.testnet network-config testnet sign-with-keychain send"
            ))
        ).chain(
            INITIAL_BALANCE_ALIASES.iter().map(|initial_balance_alias| (
                format!("near create bob.testnet --useAccount alice.testnet --{initial_balance_alias} 0.1"),
                "account create-account fund-myself bob.testnet '0.1 NEAR' autogenerate-new-keypair save-to-keychain sign-as alice.testnet network-config testnet sign-with-keychain send"
            ))
        ).chain(
            SEED_PHRASE_ALIASES.iter().map(|seed_phrase_alias| (
                format!("near create bob.testnet --{seed_phrase_alias} 'crisp clump stay mean dynamic become fashion mail bike disorder chronic sight' --useFaucet"),
                "account create-account sponsor-by-faucet-service bob.testnet use-manually-provided-seed-phrase 'crisp clump stay mean dynamic become fashion mail bike disorder chronic sight' network-config testnet create"
            ))
        ).chain(
            PUBLIC_KEY_ALIASES.iter().map(|public_key_alias| (
                format!("near create bob.testnet --useAccount alice.testnet --{public_key_alias} 78MziB9aTNsu19MHHVrfWy762S5mAqXgCB6Vgvrv9uGV --initialBalance 0.1"),
                "account create-account fund-myself bob.testnet '0.1 NEAR' use-manually-provided-public-key 78MziB9aTNsu19MHHVrfWy762S5mAqXgCB6Vgvrv9uGV sign-as alice.testnet network-config testnet sign-with-keychain send"
            ))
        ).chain(
            USE_LEDGER_PK_ALIASES.iter().map(|use_ledger_pk_alias| (
                format!("near create bob.testnet --{use_ledger_pk_alias} --useFaucet"),
                "account create-account sponsor-by-faucet-service bob.testnet use-ledger network-config testnet create"
            ))
        ) {
            let input_cmd = shell_words::split(&input).expect("Input command must be a valid shell command");
            let JsCmd::CreateAccount(create_account_args) = JsCmd::parse_from(&input_cmd) else {
                panic!("CreateAccount command was expected, but something else was parsed out from {input}");
            };
            assert_eq!(
                shell_words::join(CreateAccountArgs::to_cli_args(&create_account_args, "testnet".to_string())),
                expected_output
            );
        }
    }
}
