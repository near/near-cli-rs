use crate::js_command_match::constants::{BLOCK_ID_ALIASES, NETWORK_ID_ALIASES};

#[derive(Debug, Clone, clap::Parser)]
pub struct ViewStateArgs {
    account_id: String,
    #[clap(long)]
    prefix: Option<String>,
    #[clap(long, default_value_t = false)]
    utf8: bool,
    #[clap(long, aliases = BLOCK_ID_ALIASES)]
    block_id: Option<String>,
    #[clap(long, conflicts_with = "block_id")]
    finality: Option<String>,
    #[clap(long, aliases = NETWORK_ID_ALIASES)]
    network_id: Option<String>,
}

impl ViewStateArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);

        let mut command = vec![
            "contract".to_string(),
            "view-storage".to_string(),
            self.account_id.to_owned(),
        ];

        let output_format = if self.utf8 { "as-text" } else { "as-json" };

        if let Some(prefix) = &self.prefix {
            let prefix_type = match near_primitives::serialize::from_base64(&prefix[..]) {
                Ok(_) => "keys-start-with-bytes-as-base64".to_string(),
                Err(_) => "keys-start-with-string".to_string(),
            };

            command.push(prefix_type);
            command.push(prefix.to_string());
        } else {
            command.push("all".to_string());
        }

        command.push(output_format.to_owned());
        command.push("network-config".to_string());
        command.push(network_id);

        if self.finality.is_some() {
            command.push("now".to_string());
        } else if let Some(block_id) = &self.block_id {
            match block_id.parse::<i32>() {
                Ok(_) => {
                    command.push("at-block-height".to_string());
                }
                Err(_) => {
                    command.push("at-block-hash".to_string());
                }
            }
            command.push(block_id.to_string());
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
    fn view_state() {
        for (input, expected_output) in [
            (
                format!("near storage counter.near-examples.testnet --prefix U1RBVEU= --{} 167860267", BLOCK_ID_ALIASES[0]),
                "contract view-storage counter.near-examples.testnet keys-start-with-bytes-as-base64 'U1RBVEU=' as-json network-config testnet at-block-height 167860267"
            ),
            (
                format!("near view-state counter.near-examples.testnet --prefix U1RBVEU= --{} 167860267", BLOCK_ID_ALIASES[0]),
                "contract view-storage counter.near-examples.testnet keys-start-with-bytes-as-base64 'U1RBVEU=' as-json network-config testnet at-block-height 167860267"
            ),
            (
                format!("near view-state counter.near-examples.testnet --prefix U1RBVEU= --{} 167860267", BLOCK_ID_ALIASES[1]),
                "contract view-storage counter.near-examples.testnet keys-start-with-bytes-as-base64 'U1RBVEU=' as-json network-config testnet at-block-height 167860267"
            ),
            (
                format!("near view-state counter.near-examples.testnet --prefix STATE --utf8 --finality final --{} mainnet", NETWORK_ID_ALIASES[0]),
                "contract view-storage counter.near-examples.testnet keys-start-with-string STATE as-text network-config mainnet now"
            ),
            (
                format!("near view-state counter.near-examples.testnet --prefix STATE --utf8 --finality final --{} mainnet", NETWORK_ID_ALIASES[1]),
                "contract view-storage counter.near-examples.testnet keys-start-with-string STATE as-text network-config mainnet now"
            ),
        ] {
            let input_cmd = shell_words::split(&input).expect("Input command must be a valid shell command");
            let JsCmd::ViewState(view_state_args) = JsCmd::parse_from(&input_cmd) else {
                panic!("ViewState command was expected, but something else was parsed out from {input}");
            };
            assert_eq!(
                shell_words::join(ViewStateArgs::to_cli_args(&view_state_args, "testnet".to_string())),
                expected_output
            );
        }
    }
}
