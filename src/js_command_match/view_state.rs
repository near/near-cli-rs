#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `view-state` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct ViewStateArgs {
    contract_account_id: String,
    #[clap(long, aliases = ["network_id", "networkId"], default_value=None)]
    network_id: Option<String>,
    #[clap(long, default_value = None, conflicts_with = "block_id")]
    finality: Option<String>,
    #[clap(long, aliases = ["block_id", "blockId"], default_value = None)]
    block_id: Option<String>,
    #[clap(long, default_value = None)]
    prefix: Option<String>,
    #[clap(long, default_value_t = false)]
    utf8: bool,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl ViewStateArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let mut base_command = vec![
            "contract".to_owned(),
            "view-storage".to_owned(),
            self.contract_account_id.to_owned()
        ];
        let args: Vec<String>;
        let output_format = if self.utf8 { "as-text" } else { "as-json" };
        if self.finality.is_some() {
            args = vec![
                // "all".to_owned(),
                output_format.to_owned(),
                "network-config".to_owned(),
                self.network_id.clone().unwrap_or(network_config.to_owned()),
                "now".to_owned(),
            ];
        } else if self.block_id.is_some() {
            let block_id = self.block_id.to_owned().expect("You must provide valid blockId");
            match block_id.parse::<i32>() {
                Ok(_) => {
                    args = vec![
                        output_format.to_owned(),
                        "network-config".to_owned(),
                        self.network_id.clone().unwrap_or(network_config.to_owned()),
                        "at-block-height".to_owned(),
                        block_id,
                    ];
                }
                Err(_) => {
                    args = vec![
                        output_format.to_owned(),
                        "network-config".to_owned(),
                        self.network_id.clone().unwrap_or(network_config.to_owned()),
                        "at-block-hash".to_owned(),
                        block_id,
                    ];
                }
            }
        } else {
            args = vec![
                output_format.to_owned(),
                "network-config".to_owned(),
                self.network_id.clone().unwrap_or(network_config.to_owned()),
            ];
        }

        if self.prefix.is_some() {
            let prefix = self.prefix.to_owned().expect("You must provide valid prefix");
            let prefix_type = match near_primitives::serialize::from_base64(&prefix[..]) {
                Ok(_) => "keys-start-with-bytes-as-base64".to_owned(),
                Err(_) => "keys-start-with-string".to_owned()
            };

            base_command.extend([ 
                prefix_type,
                prefix
            ]);
        } else {
            base_command.extend([
                "all".to_owned(),
            ]);
        }

        [base_command, args].concat()
    }
}
