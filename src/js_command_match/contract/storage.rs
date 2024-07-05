use crate::js_command_match::constants::{BLOCK_ID_ALIASES, NETWORK_ID_ALIASES};

#[derive(Debug, Clone, clap::Parser)]
#[clap(alias("storage"))]
pub struct ViewStateArgs {
    account_id: String,
    #[clap(long, default_value = None)]
    prefix: Option<String>,
    #[clap(long, default_value_t = false)]
    utf8: bool,
    #[clap(long, aliases = BLOCK_ID_ALIASES, default_value = None)]
    block_id: Option<String>,
    #[clap(long, default_value = None, conflicts_with = "block_id")]
    finality: Option<String>,
    #[clap(long, aliases = NETWORK_ID_ALIASES, default_value=None)]
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

        if let Some(prefix) = self.prefix.to_owned() {
            let prefix_type = match near_primitives::serialize::from_base64(&prefix[..]) {
                Ok(_) => "keys-start-with-bytes-as-base64".to_string(),
                Err(_) => "keys-start-with-string".to_string(),
            };

            command.push(prefix_type);
            command.push(prefix);
        } else {
            command.push("all".to_string());
        }

        command.push(output_format.to_owned());
        command.push("network-config".to_string());
        command.push(network_id);

        if self.finality.is_some() {
            command.push("now".to_string());
        } else if let Some(block_id) = self.block_id.to_owned() {
            match block_id.parse::<i32>() {
                Ok(_) => {
                    command.push("at-block-height".to_string());
                }
                Err(_) => {
                    command.push("at-block-hash".to_string());
                }
            }
            command.push(block_id);
        }

        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn view_state_testnet() {
        let contract_account_id = "counter.near-examples.testnet";
        let prefix = "U1RBVEU=";
        let block_id = "167860267";

        for block_id_parameter_alias in BLOCK_ID_ALIASES {
            let view_state_args = ViewStateArgs::parse_from(&[
                "near",
                contract_account_id,
                "--prefix",
                prefix,
                &format!("--{block_id_parameter_alias}"),
                block_id,
            ]);
            let result = ViewStateArgs::to_cli_args(&view_state_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!(
                    "contract view-storage {contract_account_id} keys-start-with-bytes-as-base64 {prefix} as-json network-config testnet at-block-height {block_id}",
                )
            )
        }
    }

    #[test]
    fn view_state_mainnet() {
        let contract_account_id = "counter.near-examples.testnet";
        let prefix = "STATE";
        let finality = "final";
        let network_id = "mainnet";

        for network_id_parameter_alias in NETWORK_ID_ALIASES {
            let view_state_args = ViewStateArgs::parse_from(&[
                "near",
                contract_account_id,
                "--prefix",
                prefix,
                "--utf8",
                "--finality",
                finality,
                &format!("--{network_id_parameter_alias}"),
                network_id,
            ]);
            let result = ViewStateArgs::to_cli_args(&view_state_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!(
                    "contract view-storage {contract_account_id} keys-start-with-string {prefix} as-text network-config {network_id} now",
                )
            )
        }
    }
}
