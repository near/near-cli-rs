#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct ViewAccountSummary {
    ///What Account ID do you need to view?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    ///Select network
    network: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

impl ViewAccountSummary {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        let network_config = self.network.get_network_config(config);
        crate::common::display_account_info(
            self.account_id.clone().into(),
            network_config.clone(),
            self.network.get_block_ref(),
        )
        .await?;
        crate::common::display_access_key_list(
            self.account_id.clone().into(),
            network_config,
            self.network.get_block_ref(),
        )
        .await?;
        Ok(())
    }
}
