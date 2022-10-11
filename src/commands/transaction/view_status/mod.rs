use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct TransactionInfo {
    ///Enter the hash of the transaction you need to view
    transaction_hash: String,
    ///What is the signer account ID?
    signer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network::Network,
}

impl TransactionInfo {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        let query_view_transaction_status = self.network_config.get_network_config(config).json_rpc_client()?
            .call(near_jsonrpc_client::methods::EXPERIMENTAL_tx_status::RpcTransactionStatusRequest {
                transaction_info: near_jsonrpc_client::methods::EXPERIMENTAL_tx_status::TransactionInfo::TransactionId {
                    hash: near_primitives::hash::CryptoHash::from_str(&self.transaction_hash).unwrap(),
                    account_id: self.signer_account_id.clone().into()
                }
            })
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to fetch query for view transaction: {:?}",
                    err
                ))
            })?;
        println!("Transaction status: {:#?}", query_view_transaction_status);
        Ok(())
    }
}
