use interactive_clap::ToCliArgs;

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `legacy` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct LoginArgs {
    #[clap(long, aliases = ["wallet_url", "walletUrl"], default_value = "https://wallet.testnet.near.org")]
    wallet_url: String,
    #[clap(long, aliases = ["network_id", "networkId"], default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl LoginArgs {
    pub fn to_cli_args(&self, network_config: String) -> std::collections::VecDeque<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);

        crate::commands::CliTopLevelCommand::Account(
            crate::commands::account::CliAccountCommands {
                account_actions: Some(crate::commands::account::CliAccountActions::ImportAccount(
                    crate::commands::account::import_account::CliImportAccountCommand {
                        import_account_actions: Some(crate::commands::account::import_account::CliImportAccountActions::UsingWebWallet(
                            crate::commands::account::import_account::using_web_wallet::CliLoginFromWebWallet {
                                network_config: Some(crate::commands::account::import_account::using_web_wallet::ClapNamedArgNetworkForLoginFromWebWallet::NetworkConfig(
                                    crate::network::CliNetwork {
                                    wallet_url: None,
                                    network_name: Some(network_id),
                                })),
                            }
                        ))
                    }
                ))
            }
        ).to_cli_args()
    }
}
