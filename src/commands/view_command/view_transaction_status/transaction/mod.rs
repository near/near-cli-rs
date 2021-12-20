use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::operation_mode::online_mode::select_server::ViewTransactionCommandNetworkContext)]
pub struct TransactionType {
    pub transaction_hash: String,
    #[interactive_clap(named_arg)]
    signer: super::signer::Sender,
}

impl TransactionType {
    fn input_transaction_hash(
        _context: &super::operation_mode::online_mode::select_server::ViewTransactionCommandNetworkContext,
    ) -> color_eyre::eyre::Result<String> {
        println!();
        Ok(Input::new()
            .with_prompt("Enter the hash of the transaction you need to view")
            .interact_text()
            .unwrap())
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        self.signer
            .process(network_connection_config, self.transaction_hash)
            .await
    }
}
