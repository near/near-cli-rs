use color_eyre::eyre::Context;

use crate::common::{CallResultExt, JsonRpcClientExt, RpcQueryResponseExt};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ViewAccountSummaryContext)]
pub struct ViewAccountSummary {
    #[interactive_clap(skip_default_input_arg)]
    /// What Account ID do you need to view?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct ViewAccountSummaryContext(crate::network_view_at_block::ArgsForViewContext);

impl ViewAccountSummaryContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<ViewAccountSummary as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let account_id: near_primitives::types::AccountId = scope.account_id.clone().into();

            move |network_config, block_reference| {
                let rpc_query_response = network_config
                    .json_rpc_client()
                    .blocking_call_view_account(&account_id, block_reference.clone())
                    .wrap_err_with(|| {
                        format!(
                            "Failed to fetch query ViewAccount for <{}>",
                            &account_id
                        )
                    })?;
                let account_view = rpc_query_response.account_view()?;

                let access_key_list = network_config
                    .json_rpc_client()
                    .blocking_call_view_access_key_list(
                        &account_id,
                        block_reference.clone(),
                    )
                    .wrap_err_with(|| {
                        format!(
                            "Failed to fetch ViewAccessKeyList for {}",
                            &account_id
                        )
                    })?
                    .access_key_list_view()?;

                let contract_account_id = network_config.get_near_social_account_id_from_network()?;

                let social_db = network_config
                    .json_rpc_client()
                    .blocking_call_view_function(
                        &contract_account_id,
                        "get",
                        serde_json::json!({
                            "keys": vec![format!("{account_id}/profile/**")],
                        })
                        .to_string()
                        .into_bytes(),
                        block_reference.clone(),
                    )
                    .wrap_err_with(|| {format!("Failed to fetch query for view method: 'get {account_id}/profile/**'")})?
                    .parse_result_from_json::<crate::types::socialdb_types::SocialDb>()
                    .wrap_err_with(|| {
                        format!("Failed to parse view function call return value for {account_id}/profile.")
                    })?;

                crate::common::display_account_info(
                    &rpc_query_response.block_hash,
                    &rpc_query_response.block_height,
                    &account_id,
                    &account_view,
                    &access_key_list.keys,
                    social_db.accounts.get(&account_id)
                );

                Ok(())
            }
        });
        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.config,
            interacting_with_account_ids: vec![scope.account_id.clone().into()],
            on_after_getting_block_reference_callback,
        }))
    }
}

impl From<ViewAccountSummaryContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: ViewAccountSummaryContext) -> Self {
        item.0
    }
}

impl ViewAccountSummary {
    pub fn input_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What Account ID do you need to view?",
        )
    }
}
