pub mod generate_keypair_subcommand;

/// Collection of various low-level helpers
#[derive(Debug)]
pub struct CliArgs {
    subcommand: CliSubCommand,
}

impl CliArgs {
    fn rpc_client(&self) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client("https://rpc.testnet.near.org")
    }
}

#[derive(Debug)]
pub enum CliSubCommand {
    GenerateKeypair(generate_keypair_subcommand::GenerateKeypair),
}

impl CliArgs {
    pub async fn process(self) -> String {
        match self.subcommand {
            CliSubCommand::GenerateKeypair(generate_keypair_subcommand) => {
                generate_keypair_subcommand.process().await
            }
        }
    }
}
