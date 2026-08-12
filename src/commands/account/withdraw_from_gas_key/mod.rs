use inquire::{CustomType, Select, formatter::OptionFormatter};

use crate::common::AccessKeyInfo;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = WithdrawFromGasKeyCommandContext)]
pub struct WithdrawFromGasKeyCommand {
    #[interactive_clap(skip_default_input_arg)]
    /// Which account owns the gas key you want to withdraw from?
    owner_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subargs)]
    /// Input Withdraw GasKey details
    withdraw_gas_key_details: WithdrawFromGasKeyDetails,
}

#[derive(Debug, Clone)]
pub struct WithdrawFromGasKeyCommandContext {
    global_context: crate::GlobalContext,
    owner_account_id: near_primitives::types::AccountId,
}

impl WithdrawFromGasKeyCommandContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<WithdrawFromGasKeyCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            owner_account_id: scope.owner_account_id.clone().into(),
        })
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

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = WithdrawFromGasKeyCommandContext)]
#[interactive_clap(output_context = WithdrawFromGasKeyDetailsContext)]
pub struct WithdrawFromGasKeyDetails {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter the public key of the gas key:
    public_key: crate::types::public_key::PublicKey,

    /// How much NEAR do you want to withdraw from the gas key balance back to the account? (example: 1 NEAR or 0.5 NEAR or 10000 yoctonear)
    amount: crate::types::near_token::NearToken,

    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct WithdrawFromGasKeyDetailsContext {
    global_context: crate::GlobalContext,
    owner_account_id: near_primitives::types::AccountId,
    public_key: near_crypto::PublicKey,
    amount: crate::types::near_token::NearToken,
}

impl WithdrawFromGasKeyDetailsContext {
    pub fn from_previous_context(
        previous_context: WithdrawFromGasKeyCommandContext,
        scope: &<WithdrawFromGasKeyDetails as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            owner_account_id: previous_context.owner_account_id,
            public_key: scope.public_key.clone().into(),
            amount: scope.amount,
        })
    }
}

impl WithdrawFromGasKeyDetails {
    pub fn input_public_key_manually()
    -> color_eyre::eyre::Result<Option<crate::types::public_key::PublicKey>> {
        Ok(Some(CustomType::new("Enter a GasKey public key you want to withdraw funds from (for example, ed25519:FAXX...RUQa or ed25519:FgVF...oSWJ):").prompt()?))
    }

    pub fn input_public_key(
        context: &WithdrawFromGasKeyCommandContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::public_key::PublicKey>> {
        if context.global_context.offline {
            return Self::input_public_key_manually();
        }

        let (access_key_list, errors) = crate::common::get_public_keys_from_network_config(
            &context.global_context,
            &context.owner_account_id,
        )?;

        let access_key_list: Vec<AccessKeyInfo> = access_key_list
            .iter()
            .filter(|info| {
                matches!(
                    info.permission,
                    near_primitives::views::AccessKeyPermissionView::GasKeyFunctionCall { .. }
                        | near_primitives::views::AccessKeyPermissionView::GasKeyFullAccess { .. }
                )
            })
            .cloned()
            .collect();

        if access_key_list.is_empty() {
            for error in errors {
                println!("WARNING! {error}");
            }
            println!(
                "Automatic search of gas keys for <{}> is not possible on [{}] network(s).\nYou can enter gas key to withdraw funds from manually.",
                context.owner_account_id,
                context.global_context.config.network_names().join(", ")
            );
            return Self::input_public_key_manually();
        }

        let formatter: OptionFormatter<'_, AccessKeyInfo> = &|a| a.to_string();

        let selected_public_key = Select::new(
            "Select the GasKey you want to withdraw from:",
            access_key_list,
        )
        .with_formatter(formatter)
        .prompt()?;

        Ok(Some(selected_public_key.public_key.clone().into()))
    }
}

impl From<WithdrawFromGasKeyDetailsContext> for crate::commands::ActionContext {
    fn from(item: WithdrawFromGasKeyDetailsContext) -> Self {
        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let owner_account_id = item.owner_account_id.clone();
                let public_key = item.public_key.clone();
                let amount = item.amount;

                move |_network_config| {
                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: owner_account_id.clone(),
                        receiver_id: owner_account_id.clone(),
                        actions: vec![near_primitives::transaction::Action::WithdrawFromGasKey(
                            Box::new(near_primitives::action::WithdrawFromGasKeyAction {
                                public_key: public_key.clone(),
                                amount: amount.into(),
                            }),
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
