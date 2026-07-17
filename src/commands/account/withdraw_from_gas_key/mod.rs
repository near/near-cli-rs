#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = WithdrawFromGasKeyContext)]
pub struct WithdrawFromGasKeyCommand {
    #[interactive_clap(skip_default_input_arg)]
    /// Which account owns the gas key you want to withdraw from?
    owner_account_id: crate::types::account_id::AccountId,
    /// Enter the public key of the gas key:
    public_key: crate::types::public_key::PublicKey,
    /// How much NEAR do you want to withdraw from the gas key balance back to the account? (example: 1 NEAR or 0.5 NEAR or 10000 yoctonear)
    amount: crate::types::near_token::NearToken,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct WithdrawFromGasKeyContext {
    global_context: crate::GlobalContext,
    owner_account_id: near_kit::AccountId,
    public_key: crate::types::public_key::PublicKey,
    amount: crate::types::near_token::NearToken,
}

impl WithdrawFromGasKeyContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<WithdrawFromGasKeyCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            owner_account_id: scope.owner_account_id.clone().into(),
            public_key: scope.public_key.clone(),
            amount: scope.amount,
        })
    }
}

impl From<WithdrawFromGasKeyContext> for crate::commands::ActionContext {
    fn from(item: WithdrawFromGasKeyContext) -> Self {
        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let owner_account_id = item.owner_account_id.clone();
                let public_key = item.public_key.clone();
                let amount = item.amount;

                move |_network_config| {
                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: owner_account_id.clone(),
                        receiver_id: owner_account_id.clone(),
                        actions: vec![near_kit::Action::WithdrawFromGasKey(
                            near_kit::WithdrawFromGasKeyAction {
                                public_key: public_key.clone().into(),
                                amount: amount.into(),
                            },
                        )],
                    })
                }
            });

        Self {
            global_context: item.global_context,
            interacting_with_account_ids: vec![item.owner_account_id],
            get_prepopulated_transaction_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepopulated_unsigned_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
            sign_as_delegate_action: false,
            on_sending_delegate_action_callback: None,
        }
    }
}

impl WithdrawFromGasKeyCommand {
    pub fn input_owner_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "Which account owns the gas key you want to withdraw from?",
        )
    }
}
