use color_eyre::eyre::Context;
use serde_json::json;

use crate::common::CallResultExt;
use crate::common::{RpcResultExt, block_on, query_view_function};

use super::send_ft::input_ft_contract_account_id;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::TokensCommandsContext)]
#[interactive_clap(output_context = ViewFtBalanceContext)]
pub struct ViewFtBalance {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the ft-contract account ID?
    ft_contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct ViewFtBalanceContext(crate::network_view_at_block::ArgsForViewContext);

impl ViewFtBalanceContext {
    pub fn from_previous_context(
        previous_context: super::TokensCommandsContext,
        scope: &<ViewFtBalance as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let owner_account_id = previous_context.owner_account_id.clone();
            let ft_contract_account_id: near_kit::AccountId =
                scope.ft_contract_account_id.clone().into();
            let credentials_home_dir = previous_context.global_context.config.credentials_home_dir.clone();

            move |network_config, block_reference| {
                let ft_metadata = crate::types::ft_properties::params_ft_metadata(
                    ft_contract_account_id.clone(),
                    network_config,
                    block_reference.clone(),
                )?;

                let ft_contract = crate::types::ft_properties::FtContract {
                    ft_metadata: ft_metadata.clone(),
                    ft_contract_account_id: ft_contract_account_id.clone(),
                };

                crate::common::update_used_ft_contract_account_list(
                    &credentials_home_dir,
                    &ft_contract,
                );

                let args = serde_json::to_vec(&json!({
                    "account_id": owner_account_id.to_string(),
                    }))?;
                let call_result = get_ft_balance(network_config, &ft_contract_account_id, args, block_reference.clone())?;
                call_result.print_logs();
                let amount: String = call_result.parse_result_from_json()?;
                let fungible_token = near_kit::FtAmount::new(
                    amount.parse::<u128>()?,
                    ft_metadata.decimals,
                    ft_metadata.symbol,
                );

                println!("<{owner_account_id}> account has {fungible_token}  (FT-contract: {ft_contract_account_id})");

                Ok(())
            }
        });
        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.global_context.config,
            on_after_getting_block_reference_callback,
            interacting_with_account_ids: vec![
                scope.ft_contract_account_id.clone().into(),
                previous_context.owner_account_id,
            ],
        }))
    }
}

impl From<ViewFtBalanceContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: ViewFtBalanceContext) -> Self {
        item.0
    }
}

impl ViewFtBalance {
    pub fn input_ft_contract_account_id(
        context: &super::TokensCommandsContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        input_ft_contract_account_id(&context.global_context.config.credentials_home_dir)
    }
}

#[tracing::instrument(name = "Getting FT balance ...", skip_all, parent = None)]
pub fn get_ft_balance(
    network_config: &crate::config::NetworkConfig,
    ft_contract_account_id: &near_kit::AccountId,
    args: Vec<u8>,
    block_reference: near_kit::BlockReference,
) -> color_eyre::eyre::Result<near_kit::ViewFunctionResult> {
    tracing::info!(target: "near_teach_me", "Getting FT balance ...");
    let result = block_on(query_view_function(
            network_config.client().rpc(),
            ft_contract_account_id,
            "ft_balance_of",
            &args,
            block_reference,
        ))
        .into_eyre()
        .wrap_err_with(||{
            format!("Failed to fetch query for view method: 'ft_balance_of' (contract <{}> on network <{}>)",
                ft_contract_account_id,
                network_config.network_name
            )
        })?;
    Ok(result)
}
