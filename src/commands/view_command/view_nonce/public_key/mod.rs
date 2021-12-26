use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::operation_mode::online_mode::select_server::ViewNonceCommandNetworkContext)]
pub struct AccessKeyType {
    pub public_key: crate::types::public_key::PublicKey,
}

impl AccessKeyType {
    fn input_public_key(
        _context: &super::operation_mode::online_mode::select_server::ViewNonceCommandNetworkContext,
    ) -> color_eyre::eyre::Result<crate::types::public_key::PublicKey> {
        Ok(Input::new()
            .with_prompt("Enter a public key for this access key")
            .interact_text()?)
    }

    fn rpc_client(self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    pub async fn process(
        self,
        account_id: near_primitives::types::AccountId,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        let public_key = self.public_key.clone();
        let online_signer_access_key_response = self
            .rpc_client(network_connection_config.rpc_url().as_str())
            .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::ViewAccessKey {
                    account_id,
                    public_key: public_key.clone().into(),
                },
            })
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to fetch public key information for nonce: {:?}",
                    err
                ))
            })?;
        let current_nonce =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKey(
                online_signer_access_key,
            ) = online_signer_access_key_response.kind
            {
                online_signer_access_key.nonce
            } else {
                return Err(color_eyre::Report::msg(format!("Error current_nonce")));
            };
        println!(
            "\ncurrent nonce: {}  for a public key: {}",
            current_nonce, public_key
        );
        Ok(())
    }
}
