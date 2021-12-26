#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = ())]
#[interactive_clap(output_context = OfflineArgsContext)]
pub struct OfflineArgs {
    #[interactive_clap(named_arg)]
    ///Specify owner account
    owner_account: super::super::sender::Sender,
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

impl From<OfflineArgsContext> for super::AddSubAccountCommandNetworkContext {
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
        self.owner_account
            .process(prepopulated_unsigned_transaction, selected_server_url)
            .await
    }
}
