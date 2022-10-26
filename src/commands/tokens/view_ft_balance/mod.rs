use serde_json::json;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct ViewFtBalance {
    ///What is the ft-contract account ID?
    ft_contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

impl ViewFtBalance {
    pub async fn process(
        &self,
        config: crate::config::Config,
        owner_account_id: near_primitives::types::AccountId,
    ) -> crate::CliResult {
        let (decimals, symbol) = super::params_ft_metadata(
            config.clone(),
            self.ft_contract_account_id.clone(),
            self.network_config.clone(),
        )
        .await?;

        let method_name = "ft_balance_of".to_string();
        let args = json!({
            "account_id": owner_account_id.to_string(),
        })
        .to_string()
        .into_bytes();
        let query_view_method_response = self
            .network_config
            .get_network_config(config)
            .json_rpc_client()?
            .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: self.network_config.get_block_ref(),
                request: near_primitives::views::QueryRequest::CallFunction {
                    account_id: self.ft_contract_account_id.clone().into(),
                    method_name,
                    args: near_primitives::types::FunctionArgs::from(args),
                },
            })
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!("Failed to fetch query for view method: {:?}", err))
            })?;
        let call_result =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::CallResult(result) =
                query_view_method_response.kind
            {
                result.result
            } else {
                return Err(color_eyre::Report::msg("Error call result".to_string()));
            };

        let serde_call_result = if call_result.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_slice(&call_result)
                .map_err(|err| color_eyre::Report::msg(format!("serde json: {:?}", err)))?
        };
        let amount: String = serde_json::from_value(serde_call_result.clone()).unwrap();
        let amount = amount.parse::<u128>().unwrap();
        let amount_fmt = {
            if amount == 0 {
                format!("0 {}", symbol)
            } else if (amount % 10u128.pow(decimals as u32)) == 0 {
                format!("{} {}", amount / 10u128.pow(decimals as u32), symbol,)
            } else {
                format!(
                    "{}.{} {}",
                    amount / 10u128.pow(decimals as u32),
                    format!("{:0>24}", amount % 10u128.pow(decimals as u32)).trim_end_matches('0'),
                    symbol
                )
            }
        };

        println!(
            "\n<{}> account has {}  (FT-contract: {})",
            owner_account_id, amount_fmt, self.ft_contract_account_id
        );
        Ok(())
    }
}
