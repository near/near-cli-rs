use std::str::FromStr;

use inquire::{CustomType, Select};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::ContractContext)]
#[interactive_clap(output_context = DepositArgsContext)]
pub struct DepositArgs {
    /// Which account ID do you want to add a deposit to?
    receiver_account_id: crate::types::account_id::AccountId,
    /// Enter the amount to deposit into the storage (example: 10NEAR or 0.5near or 10000yoctonear)
    deposit: crate::common::NearBalance,
    #[interactive_clap(named_arg)]
    /// What is the signer account ID?
    sign_as: SignerAccountId,
}

#[derive(Clone)]
pub struct DepositArgsContext {
    config: crate::config::Config,
    get_contract_account_id: super::GetContractAccountID,
    receiver_account_id: near_primitives::types::AccountId,
    deposit: crate::common::NearBalance,
}

impl DepositArgsContext {
    pub fn from_previous_context(
        previous_context: super::ContractContext,
        scope: &<DepositArgs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            get_contract_account_id: previous_context.get_contract_account_id,
            receiver_account_id: scope.receiver_account_id.clone().into(),
            deposit: scope.deposit.clone(),
        })
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = DepositArgsContext)]
#[interactive_clap(output_context = SignerAccountIdContext)]
pub struct SignerAccountId {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the signer account ID?
    signer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct SignerAccountIdContext(crate::commands::ActionContext);

impl SignerAccountIdContext {
    pub fn from_previous_context(
        previous_context: DepositArgsContext,
        scope: &<SignerAccountId as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let signer = scope.signer_account_id.clone();
        let signer_account_id = scope.signer_account_id.clone();
        let deposit = previous_context.deposit.clone();
        let receiver_account_id = previous_context.receiver_account_id.clone();
        let get_contract_account_id = previous_context.get_contract_account_id.clone();

        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::sync::Arc::new(move |prepopulated_unsigned_transaction, network_config| {
                let contract_account_id = get_contract_account_id(network_config)?;
                prepopulated_unsigned_transaction.signer_id = signer_account_id.clone().into();
                prepopulated_unsigned_transaction.receiver_id = contract_account_id;
                prepopulated_unsigned_transaction.actions =
                    vec![near_primitives::transaction::Action::FunctionCall(
                        near_primitives::transaction::FunctionCallAction {
                            method_name: "storage_deposit".to_string(),
                            args: serde_json::json!({
                                "account_id": &previous_context.receiver_account_id.clone()
                            })
                            .to_string()
                            .into_bytes(),
                            gas: crate::common::NearGas::from_str("300 TeraGas")
                                .unwrap()
                                .inner,
                            deposit: previous_context.deposit.to_yoctonear(),
                        },
                    )];
                Ok(())
            });

        let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new(
            move |outcome_view, network_config| {
                        let contract_account_id = (previous_context.get_contract_account_id)(network_config)?;

                if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = outcome_view.status {
                    eprintln!(
                        "<{signer}> has successfully added a deposit of {deposit} to <{receiver_account_id}> on contract <{contract_account_id}>.",
                    );
                }
                Ok(())
            },
        );

        Ok(Self(crate::commands::ActionContext {
            config: previous_context.config.clone(),
            on_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
            on_after_sending_transaction_callback,
        }))
    }
}

impl From<SignerAccountIdContext> for crate::commands::ActionContext {
    fn from(item: SignerAccountIdContext) -> Self {
        item.0
    }
}

impl SignerAccountId {
    fn input_signer_account_id(
        context: &DepositArgsContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        loop {
            let signer_account_id: crate::types::account_id::AccountId =
                CustomType::new(" What is the signer account ID?")
                    .with_default(context.receiver_account_id.clone().into())
                    .prompt()?;
            if !crate::common::is_account_exist(
                &context.config.network_connection,
                signer_account_id.clone().into(),
            ) {
                eprintln!("\nThe account <{signer_account_id}> does not yet exist.");
                #[derive(strum_macros::Display)]
                enum ConfirmOptions {
                    #[strum(to_string = "Yes, I want to enter a new account name.")]
                    Yes,
                    #[strum(to_string = "No, I want to use this account name.")]
                    No,
                }
                let select_choose_input = Select::new(
                    "Do you want to enter another receiver account id?",
                    vec![ConfirmOptions::Yes, ConfirmOptions::No],
                )
                .prompt()?;
                if let ConfirmOptions::No = select_choose_input {
                    return Ok(Some(signer_account_id));
                }
            } else {
                return Ok(Some(signer_account_id));
            }
        }
    }
}
