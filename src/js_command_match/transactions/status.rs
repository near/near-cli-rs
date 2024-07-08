use crate::js_command_match::constants::NETWORK_ID_ALIASES;

#[derive(Debug, Clone, clap::Parser)]
pub struct TxStatusArgs {
    hash: String,
    #[clap(long, aliases = NETWORK_ID_ALIASES)]
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
            self.hash.to_owned(),
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
        let transaction_hash = "4HxfV69Brk7fJd3NC63ti2H3QCgwiUiMAPvwNmGWbVXo";
        let state_args = TxStatusArgs::parse_from(&["near", transaction_hash]);
        let result = TxStatusArgs::to_cli_args(&state_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            format!("transaction view-status {transaction_hash} network-config testnet")
        )
    }

    #[test]
    fn tx_status_mainnet() {
        let transaction_hash = "4HxfV69Brk7fJd3NC63ti2H3QCgwiUiMAPvwNmGWbVXo";
        let network_id = "mainnet";

        for network_id_parameter_alias in NETWORK_ID_ALIASES {
            let state_args = TxStatusArgs::parse_from(&[
                "near",
                transaction_hash,
                &format!("--{network_id_parameter_alias}"),
                network_id,
            ]);
            let result = TxStatusArgs::to_cli_args(&state_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!("transaction view-status {transaction_hash} network-config {network_id}")
            )
        }
    }
}
