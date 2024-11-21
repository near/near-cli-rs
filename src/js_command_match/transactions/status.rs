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
    use super::super::super::JsCmd;
    use super::*;
    use clap::Parser;

    #[test]
    fn tx_status() {
        for (input, expected_output) in [
            (
                "near tx-status 4HxfV69Brk7fJd3NC63ti2H3QCgwiUiMAPvwNmGWbVXo".to_string(),
                "transaction view-status 4HxfV69Brk7fJd3NC63ti2H3QCgwiUiMAPvwNmGWbVXo network-config testnet"
            ),
            (
                format!("near tx-status 4HxfV69Brk7fJd3NC63ti2H3QCgwiUiMAPvwNmGWbVXo --{} testnet", NETWORK_ID_ALIASES[0]),
                "transaction view-status 4HxfV69Brk7fJd3NC63ti2H3QCgwiUiMAPvwNmGWbVXo network-config testnet"
            ),
            (
                format!("near tx-status 4HxfV69Brk7fJd3NC63ti2H3QCgwiUiMAPvwNmGWbVXo --{} mainnet", NETWORK_ID_ALIASES[1]),
                "transaction view-status 4HxfV69Brk7fJd3NC63ti2H3QCgwiUiMAPvwNmGWbVXo network-config mainnet"
            ),
        ] {
            let input_cmd = shell_words::split(&input).expect("Input command must be a valid shell command");
            let JsCmd::TxStatus(tx_status_args) = JsCmd::parse_from(&input_cmd) else {
                panic!("TxStatus command was expected, but something else was parsed out from {input}");
            };
            assert_eq!(
                shell_words::join(TxStatusArgs::to_cli_args(&tx_status_args, "testnet".to_string())),
                expected_output
            );
        }
    }
}
