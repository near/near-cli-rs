#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `view-state` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct ViewStateArgs {
    contract_account_id: String,
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
        if self.prefix.is_none() {
            let output_format = if self.utf8 { "as-text" } else { "as-json" };
            if self.finality.is_some() {
                vec![
                    "contract".to_owned(),
                    "view-storage".to_owned(),
                    self.contract_account_id.to_owned(),
                    "all".to_owned(),
                    output_format.to_owned(),
                    "network-config".to_owned(),
                    network_config,
                    "now".to_owned(),
                ]
            } else {
                vec![
                    "contract".to_owned(),
                    "view-storage".to_owned(),
                    self.contract_account_id.to_owned(),
                    "all".to_owned(),
                    output_format.to_owned(),
                    "network-config".to_owned(),
                    network_config,
                ]
            }
        } else {
            vec![
                "contract".to_owned(),
                "view-storage".to_owned(),
                self.contract_account_id.to_owned(),
            ]
        }
    }
}
