use color_eyre::eyre::WrapErr;
use tracing_indicatif::span_ext::IndicatifSpanExt;

use crate::common::{CallResultExt, JsonRpcClientExt};

const STORAGE_COST_PER_BYTE: u128 = 10u128.pow(19);

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::ContractContext)]
#[interactive_clap(output_context = AccountContext)]
pub struct Account {
    #[interactive_clap(skip_default_input_arg)]
    /// What is your account ID?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct AccountContext(crate::network_view_at_block::ArgsForViewContext);

impl AccountContext {
    pub fn from_previous_context(
        previous_context: super::ContractContext,
        scope: &<Account as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback =
            std::sync::Arc::new({
                let account_id = scope.account_id.clone();

                move |network_config, block_reference| {
                    let contract_account_id = (previous_context.get_contract_account_id)(network_config)?;

                    let storage_balance =
                        get_storage_balance(
                            network_config,
                            &contract_account_id,
                            &account_id,
                            block_reference
                        )?;
                    eprintln!("storage balance for <{account_id}>:");
                    eprintln!(" {:<13} {:>10}   ({} [{:>28} yoctoNEAR])",
                        "available:",
                        bytesize::ByteSize(u64::try_from(storage_balance.available / STORAGE_COST_PER_BYTE).unwrap()),
                        near_token::NearToken::from_yoctonear(storage_balance.available),
                        storage_balance.available
                    );
                    eprintln!(" {:<13} {:>10}   ({} [{:>28} yoctoNEAR])",
                        "total:",
                        bytesize::ByteSize(u64::try_from(storage_balance.total / STORAGE_COST_PER_BYTE).unwrap()),
                        near_token::NearToken::from_yoctonear(storage_balance.total),
                        storage_balance.total
                    );

                    Ok(())
                }
            });

        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.global_context.config,
            interacting_with_account_ids: vec![scope.account_id.clone().into()],
            on_after_getting_block_reference_callback,
        }))
    }
}

impl From<AccountContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: AccountContext) -> Self {
        item.0
    }
}

impl Account {
    pub fn input_account_id(
        context: &super::ContractContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is your account ID?",
        )
    }
}

#[tracing::instrument(name = "Getting storage balance for", skip_all)]
fn get_storage_balance(
    network_config: &crate::config::NetworkConfig,
    contract_account_id: &near_primitives::types::AccountId,
    account_id: &crate::types::account_id::AccountId,
    block_reference: &near_primitives::types::BlockReference,
) -> color_eyre::eyre::Result<near_socialdb_client::StorageBalance> {
    tracing::Span::current().pb_set_message(account_id.as_ref());
    network_config
        .json_rpc_client()
        .blocking_call_view_function(
            contract_account_id,
            "storage_balance_of",
            serde_json::to_vec(&serde_json::json!({
                "account_id": account_id.to_string(),
            }))?,
            block_reference.clone(),
        )
        .wrap_err_with(|| {
            format!("Failed to fetch query for view method: 'storage_balance_of' (contract <{}> on network <{}>)",
                contract_account_id,
                network_config.network_name
            )
        })?
        .parse_result_from_json::<near_socialdb_client::StorageBalance>()
        .wrap_err("Failed to parse return value of view function call for StorageBalance.")
}
