use crate::common::{CallResultExt, JsonRpcClientExt};
use color_eyre::eyre::WrapErr;

const STORAGE_COST_PER_BYTE: u128 = 10u128.pow(19);

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::ContractContext)]
#[interactive_clap(output_context = AccountContext)]
pub struct Account {
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
                let contract_account_id = previous_context.contract_account_id;
                let account_id = scope.account_id.clone();

                move |network_config, block_reference| {
                    let storage_balance = network_config
                        .json_rpc_client()
                        .blocking_call_view_function(
                            &contract_account_id,
                            "storage_balance_of",
                            serde_json::json!({
                                "account_id": account_id.to_string(),
                            })
                            .to_string()
                            .into_bytes(),
                            block_reference.clone(),
                        )
                        .wrap_err_with(|| {
                            "Failed to fetch query for view method: 'storage_balance_of'"
                        })?
                        .parse_result_from_json::<crate::common::StorageBalance>()
                        .wrap_err_with(|| {
                            "Failed to parse return value of view function call for StorageBalance."
                        })?;
                    eprintln!("storage balance for <{account_id}>:");
                    eprintln!(" {:<13} {:>10}   ({} [{:>28} yoctoNEAR])",
                        "available:",
                        bytesize::ByteSize(u64::try_from(storage_balance.available / STORAGE_COST_PER_BYTE).unwrap()),
                        crate::common::NearBalance::from_yoctonear(storage_balance.available),
                        storage_balance.available
                    );
                    eprintln!(" {:<13} {:>10}   ({} [{:>28} yoctoNEAR])",
                        "total:",
                        bytesize::ByteSize(u64::try_from(storage_balance.total / STORAGE_COST_PER_BYTE).unwrap()),
                        crate::common::NearBalance::from_yoctonear(storage_balance.total),
                        storage_balance.total
                    );

                    Ok(())
                }
            });

        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.config,
            on_after_getting_block_reference_callback,
        }))
    }
}

impl From<AccountContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: AccountContext) -> Self {
        item.0
    }
}
