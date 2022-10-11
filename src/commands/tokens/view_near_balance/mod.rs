#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct ViewNearBalance {
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

impl ViewNearBalance {
    pub async fn process(
        &self,
        config: crate::config::Config,
        owner_account_id: near_primitives::types::AccountId,
    ) -> crate::CliResult {
        let account_transfer_allowance = crate::common::get_account_transfer_allowance(
            self.network_config.get_network_config(config),
            owner_account_id,
            self.network_config.get_block_ref(),
        )
        .await?;
        println! {"{}", &account_transfer_allowance};
        Ok(())
    }
}
