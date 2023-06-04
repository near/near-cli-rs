use inquire::{CustomType, Select};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = DeleteAccountContext)]
pub struct DeleteAccount {
    /// What Account ID to be deleted?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Enter the beneficiary ID to delete this account ID
    beneficiary: BeneficiaryAccount,
}

#[derive(Debug, Clone)]
pub struct DeleteAccountContext {
    global_context: crate::GlobalContext,
    account_id: near_primitives::types::AccountId,
}

impl DeleteAccountContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<DeleteAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            account_id: scope.account_id.clone().into(),
        })
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = DeleteAccountContext)]
#[interactive_clap(output_context = BeneficiaryAccountContext)]
pub struct BeneficiaryAccount {
    #[interactive_clap(skip_default_input_arg)]
    /// Specify a beneficiary:
    beneficiary_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct BeneficiaryAccountContext {
    global_context: crate::GlobalContext,
    account_id: near_primitives::types::AccountId,
    beneficiary_account_id: near_primitives::types::AccountId,
}

impl BeneficiaryAccountContext {
    pub fn from_previous_context(
        previous_context: DeleteAccountContext,
        scope: &<BeneficiaryAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            account_id: previous_context.account_id,
            beneficiary_account_id: scope.beneficiary_account_id.clone().into(),
        })
    }
}

impl From<BeneficiaryAccountContext> for crate::commands::ActionContext {
    fn from(item: BeneficiaryAccountContext) -> Self {
        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::sync::Arc::new(move |_network_config| {
                Ok(crate::commands::PrepopulatedTransaction {
                    signer_id: item.account_id.clone(),
                    receiver_id: item.account_id.clone(),
                    actions: vec![near_primitives::transaction::Action::DeleteAccount(
                        near_primitives::transaction::DeleteAccountAction {
                            beneficiary_id: item.beneficiary_account_id.clone(),
                        },
                    )],
                })
            });
        Self {
            global_context: item.global_context,
            on_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }
    }
}

impl BeneficiaryAccount {
    pub fn input_beneficiary_account_id(
        context: &DeleteAccountContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        let beneficiary_account_id: crate::types::account_id::AccountId =
            CustomType::new("What is the beneficiary account ID?").prompt()?;

        if context.global_context.offline {
            return Ok(Some(beneficiary_account_id));
        }

        #[derive(derive_more::Display)]
        enum ConfirmOptions {
            #[display(
                fmt = "Yes, I want to check if account <{}> exists. (It is free of charge, and only requires Internet access)",
                account_id
            )]
            Yes {
                account_id: crate::types::account_id::AccountId,
            },
            #[display(fmt = "No, I know this account exists and want to continue.")]
            No,
        }
        let select_choose_input =
            Select::new("\nDo you want to check the existence of the specified account so that you don’t waste tokens with sending a transaction that won't succeed?",
                vec![ConfirmOptions::Yes{account_id: beneficiary_account_id.clone()}, ConfirmOptions::No],
                )
                .prompt()?;
        let account_id = if let ConfirmOptions::Yes { mut account_id } = select_choose_input {
            loop {
                if crate::common::find_network_where_account_exist(
                    &context.global_context,
                    account_id.clone().into(),
                )
                .is_none()
                {
                    eprintln!("\nHeads up! You will only spend tokens if you specify the account <{account_id}> as the beneficiary, because no such account exists.");
                    if !crate::common::ask_if_different_account_id_wanted()? {
                        break account_id;
                    };
                };
                account_id = CustomType::new("What is the beneficiary account ID?").prompt()?;
            }
        } else {
            beneficiary_account_id
        };
        Ok(Some(account_id))
    }
}
