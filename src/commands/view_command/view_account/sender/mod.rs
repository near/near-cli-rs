use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::operation_mode::online_mode::select_server::ViewAccountSummaryCommandNetworkContext)]
pub struct Sender {
    pub account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    selected_block_id: super::block_id::BlockId,
}

impl Sender {
    pub fn input_account_id(
        _context: &super::operation_mode::online_mode::select_server::ViewAccountSummaryCommandNetworkContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        println!();
        Ok(Input::new()
            .with_prompt("What Account ID do you need to view?")
            .interact_text()
            .unwrap())
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        self.selected_block_id
            .process(self.account_id.into(), network_connection_config)
            .await
    }
}
