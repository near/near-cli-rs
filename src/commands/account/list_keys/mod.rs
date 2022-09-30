#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct ViewListKeys {
    ///What Account ID do you need to view?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

impl ViewListKeys {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        crate::common::display_access_key_list(
            self.account_id.clone().into(),
            self.network.get_network_config(config),
            self.network.get_block_ref(),
        )
        .await?;
        Ok(())
    }
}
