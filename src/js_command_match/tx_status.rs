use crate::js_command_match::constants::{
    ACCOUNT_ID_ALIASES,
    NETWORK_ID_ALIASES,
};

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `tx-status` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct TxStatusArgs {
    transaction_hash: String,
    #[clap(long, aliases = ACCOUNT_ID_ALIASES, default_value = "near")]
    account_id: String,
    #[clap(long, aliases = NETWORK_ID_ALIASES, default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl TxStatusArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);

        let command = vec![
            "transaction".to_string(),
            "view-status".to_string(),
            self.transaction_hash.to_owned(),
            "network-config".to_string(),
            network_id,
        ];

        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn tx_status_testnet() {
        let account_id = "relay.aurora";
        let transaction_hash = "4HxfV69Brk7fJd3NC63ti2H3QCgwiUiMAPvwNmGWbVXo";
        let state_args = TxStatusArgs::parse_from(&[
            "near",
            transaction_hash,
            account_id
        ]);
        let result = TxStatusArgs::to_cli_args(&state_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            format!(
                "transaction view-status {} network-config testnet",
                transaction_hash
            )
        )
    }

    #[test]
    fn tx_status_mainnet() {
        let transaction_hash = "4HxfV69Brk7fJd3NC63ti2H3QCgwiUiMAPvwNmGWbVXo";
        let network_id = "mainnet";

        for i in 0..NETWORK_ID_ALIASES.len() {
            let network_id_parameter_alias = &format!("--{}", &NETWORK_ID_ALIASES[i]);
            let state_args = TxStatusArgs::parse_from(&[
                "near",
                transaction_hash,
                network_id_parameter_alias,
                network_id
            ]);
            let result = TxStatusArgs::to_cli_args(&state_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!(
                    "transaction view-status {} network-config {}",
                    transaction_hash,
                    network_id
                )
            )
        }
    }
}