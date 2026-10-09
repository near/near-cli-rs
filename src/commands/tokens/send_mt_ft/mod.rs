use color_eyre::eyre::ContextCompat;

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
            return Err(color_eyre::Report::msg(format!(
                "You are currently using offline mode.\nIn offline mode, it is not possible to process the token ID <{token_id}> (metadata regarding the symbol and the number of decimal places is missing).\nTherefore, please connect to the internet and use this command again to retrieve the token metadata.",
            )));
        }

        let network_config = crate::common::find_network_where_account_exist(
            &previous_context.global_context,
            mt_contract.clone(),
        )?
        .wrap_err_with(|| format!("Contract <{mt_contract}> does not exist in networks"))?;

        let mt_ft_metadata = token_id.get_mt_ft_metadata(&network_config)?;

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

    Ok(crate::commands::PrepopulatedTransaction {
        signer_id: signer_id.clone(),
        receiver_id: mt_contract.clone(),
        actions: vec![action_mt_transfer.clone()],
    })
}

#[cfg(test)]
mod tests {
    use super::get_prepopulated_transaction;
    use crate::types::{
        ft_properties::FungibleToken, mt_ft_properties::MtFtTransfer, near_token::NearToken,
    };
    use std::str::FromStr;

    fn account_id(value: &str) -> near_primitives::types::AccountId {
        near_primitives::types::AccountId::from_str(value).unwrap()
    }

    fn assert_mt_transfer_action(memo: &str, expected_memo: Option<&str>) {
        let mt_contract = account_id("intents.near");
        let receiver_account_id = account_id("receiver.near");
        let signer_id = account_id("sender.near");
        let token_id = "nep141:wrap.near".to_string();
        let amount = FungibleToken::from_params_ft(123_456_789, 6, "USDC".to_string());
        let deposit = NearToken::from_yoctonear(1);
        let gas = crate::common::NearGas::from_tgas(100);

        let transaction = get_prepopulated_transaction(
            &mt_contract,
            &receiver_account_id,
            token_id.clone(),
            &signer_id,
            &amount,
            memo,
            deposit,
            gas,
        )
        .unwrap();

        assert_eq!(transaction.signer_id, signer_id);
        assert_eq!(transaction.receiver_id, mt_contract);
        assert_eq!(transaction.actions.len(), 1);

        let action = match &transaction.actions[0] {
            near_primitives::transaction::Action::FunctionCall(action) => action,
            other => panic!("Expected FunctionCall action, got {other:?}"),
        };

        assert_eq!(action.method_name, "mt_transfer");
        assert_eq!(
            action.gas,
            near_primitives::gas::Gas::from_gas(gas.as_gas())
        );
        assert_eq!(
            action.deposit,
            near_token::NearToken::from_yoctonear(deposit.as_yoctonear())
        );

        let transfer: MtFtTransfer = serde_json::from_slice(&action.args).unwrap();
        assert_eq!(transfer.receiver_id, receiver_account_id);
        assert_eq!(transfer.token_id, token_id);
        assert_eq!(transfer.amount, amount.amount());
        assert_eq!(transfer.approval, None);
        assert_eq!(transfer.memo.as_deref(), expected_memo);
    }

    #[test]
    fn creates_mt_transfer_action_with_expected_arguments() {
        assert_mt_transfer_action("transfer note", Some("transfer note"));
    }

    #[test]
    fn omits_empty_memo_from_mt_transfer_action() {
        assert_mt_transfer_action("", None);
    }
}
