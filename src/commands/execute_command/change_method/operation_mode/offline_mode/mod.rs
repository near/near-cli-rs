#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = ())]
#[interactive_clap(output_context = OfflineArgsContext)]
pub struct OfflineArgs {
    #[interactive_clap(named_arg)]
    ///Specify a contract
    contract: super::super::contract::Contract,
}

struct OfflineArgsContext {}

impl OfflineArgsContext {
    fn from_previous_context(
        _previous_context: (),
        _scope: &<OfflineArgs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Self {
        Self {}
    }
}

impl From<OfflineArgsContext> for super::ExecuteChangeMethodCommandNetworkContext {
    fn from(_: OfflineArgsContext) -> Self {
        Self {
            connection_config: None,
        }
    }
}

impl OfflineArgs {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        let selected_server_url = None;
        self.contract
            .process(prepopulated_unsigned_transaction, selected_server_url)
            .await
    }
}
