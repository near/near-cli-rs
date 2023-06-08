#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = LoginFromWebWalletContext)]
pub struct LoginFromWebWallet {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    wallet_url: Option<crate::types::url::Url>,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}
