#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `call` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct CallArgs {
    contract_account_id: String,
    method_name: String,
    #[clap(long, default_value_t = false)]
    base64: bool,
    args: String,
    #[clap(long, aliases = ["account_id", "accountId"])]
    account_id: String,
    #[clap(long, default_value_t = 30_000_000_000_000)]
    gas: u64,
    #[clap(long, default_value = "0")]
    deposit: String,
    #[clap(long, aliases = ["network_id", "networkId"], default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl CallArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config.to_owned());

        let mut command = vec![
            "account".to_owned(),
            "add-key".to_owned(),
            self.account_id.to_owned(),
            "contract".to_owned(),
            "call-function".to_owned(),
            "as-transaction".to_owned(),
            self.contract_account_id.to_owned(),
            self.method_name.to_owned(),
        ];

        if self.base64 {
            command.push("base64-args".to_owned());
        } else {
            command.push("json-args".to_owned());            
        };

        
        command.push(self.args.to_owned());
        command.push("prepaid-gas".to_owned());
        command.push(format!("{} TeraGas", self.gas / 1_000_000_000_000));
        command.push("attached-deposit".to_owned());
        command.push(format!("{} NEAR", self.deposit));
        command.push("sign-as".to_owned());
        command.push(self.account_id.to_owned());
        command.push("network-config".to_owned());
        command.push(network_id);
        command.push("sign-with-keychain".to_owned());
        command.push("send".to_owned());

        command
    }
}
