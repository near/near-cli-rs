use color_eyre::eyre::{Context, ContextCompat};
use color_eyre::owo_colors::OwoColorize;

use crate::types::mt_ft_inventory::{nep141_mt_ft_metadata, nep245_mt_ft_metadata};
use crate::types::mt_ft_properties::{IntentsTokenId, MtFtMetadata, TokenId};

mod amount_mt_ft;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::TokensCommandsContext)]
#[interactive_clap(output_context = IntentContractIdContext)]
pub struct IntentContractId {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the Intent Contract Identifier?
    mt_contract: crate::types::account_id::AccountId,
    /// Which multi-token do you want to transfer? (e.g., nep141:usdt.tether-token.near)
    token_id: IntentsTokenId,
    #[interactive_clap(subargs)]
    /// Specify sending MT-FT command parameters:
    amount_mt_ft: self::amount_mt_ft::AmountMtFt,
}

#[derive(Debug, Clone)]
pub struct IntentContractIdContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    mt_contract: near_primitives::types::AccountId,
    token_id: IntentsTokenId,
    mt_ft_metadata: MtFtMetadata,
}

impl IntentContractIdContext {
    pub fn from_previous_context(
        previous_context: super::TokensCommandsContext,
        scope: &<IntentContractId as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let token_id = scope.token_id.clone();
        let mt_contract: near_primitives::types::AccountId = scope.mt_contract.clone().into();

        if previous_context.global_context.offline {
            return Err(color_eyre::Report::msg(
                "Cannot enter an exact MT-FT amount to transfer in offline mode when MT-FT metadata does not contain symbol information, because the prompt for entering MT-FT amount will not contain token symbol information, and the amount of decimals in the entered amount will not be validated against the decimals specified in MT-FT metadata.",
            ));
        }

        let network_config = crate::common::find_network_where_account_exist(
            &previous_context.global_context,
            mt_contract.clone(),
        )?
        .wrap_err_with(|| format!("Contract <{mt_contract}> does not exist in networks"))?;

        let mt_ft_metadata = match &token_id {
            IntentsTokenId::Nep141(nep141_contract_id) => tokio::runtime::Runtime::new()
                .wrap_err("Failed to create a new tokio runtime")?
                .block_on(async {
                    nep141_mt_ft_metadata(
                        nep141_contract_id,
                        &network_config,
                        near_primitives::types::Finality::Final.into(),
                    )
                    .await
                })
                .wrap_err_with(|| {
                    format!(
                        "Failed to get MT-FT metadata for token ID <{token_id}> on network <{}>",
                        network_config.network_name
                    )
                })?,
            IntentsTokenId::Nep245(nep245_contract_id, token) => tokio::runtime::Runtime::new()
                .wrap_err("Failed to create a new tokio runtime")?
                .block_on(async {
                    nep245_mt_ft_metadata(
                        nep245_contract_id,
                        token.clone(),
                        &network_config,
                        near_primitives::types::Finality::Final.into(),
                    )
                    .await
                })
                .wrap_err_with(|| {
                    format!(
                        "Failed to get MT-FT metadata for token ID <{token_id}> on network <{}>",
                        network_config.network_name
                    )
                })?,
        };

        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.owner_account_id,
            mt_contract,
            token_id,
            mt_ft_metadata,
        })
    }
}

impl IntentContractId {
    pub fn input_mt_contract(
        context: &super::TokensCommandsContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the account ID of a contract that supports the multi-token (MT) standard for a fungible token (FT)?",
        )
    }
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(
    name = "Creating a pre-populated transaction for signature ...",
    skip_all
)]
pub fn get_prepopulated_transaction(
    offline_mode: bool,
    network_config: &crate::config::NetworkConfig,
    mt_contract: &near_primitives::types::AccountId,
    receiver_account_id: &near_primitives::types::AccountId,
    token_id: TokenId,
    signer_id: &near_primitives::types::AccountId,
    amount_ft: &crate::types::ft_properties::FungibleToken,
    memo: &str,
    deposit: crate::types::near_token::NearToken,
    gas: crate::common::NearGas,
) -> color_eyre::eyre::Result<crate::commands::PrepopulatedTransaction> {
    tracing::info!(target: "near_teach_me", "Creating a pre-populated transaction for signature ...");
    let args_mt_transfer = serde_json::to_vec(&crate::types::mt_ft_properties::MtFtTransfer {
        receiver_id: receiver_account_id.clone(),
        token_id,
        amount: amount_ft.amount(),
        approval: None, // The approval option is not currently in use, but it can be added in the future if needed.
        memo: if memo.is_empty() {
            None
        } else {
            Some(memo.to_string())
        },
    })?;

    let action_mt_transfer = near_primitives::transaction::Action::FunctionCall(Box::new(
        near_primitives::transaction::FunctionCallAction {
            method_name: "mt_transfer".to_string(),
            args: args_mt_transfer,
            gas: near_primitives::gas::Gas::from_gas(gas.as_gas()),
            deposit: deposit.into(),
        },
    ));

    if offline_mode {
        tracing::warn!(
            target: "near_teach_me",
            "{}{}",
            "Offline mode is enabled.".red(),
            crate::common::indent_payload(&format!("\n{}",
                format!("The transaction will be created with only the mt_transfer action\nwithout checking whether the mt-contrat account is registered on the <{}> network.\nMake sure that you have enough funds in your account to cover both the mt_transfer action and\nthe deposit if the recipient account is not registered on mt-contract <{}>.",
                    network_config.network_name,
                    mt_contract
                ).yellow()
            ))
        );
        return Ok(crate::commands::PrepopulatedTransaction {
            signer_id: signer_id.clone(),
            receiver_id: mt_contract.clone(),
            actions: vec![action_mt_transfer.clone()],
        });
    };

    Ok(crate::commands::PrepopulatedTransaction {
        signer_id: signer_id.clone(),
        receiver_id: mt_contract.clone(),
        actions: vec![action_mt_transfer.clone()],
    })
}
