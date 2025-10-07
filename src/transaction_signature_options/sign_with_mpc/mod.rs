use inquire::CustomType;

use crate::commands::TransactionContext;
mod mpc_sign_with;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = SignMpcContext)]
pub struct SignMpc {
    #[interactive_clap(skip_default_input_arg)]
    /// Smart contract address that will sign MPC
    smart_contract_address: crate::types::account_id::AccountId,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    /// Admin account for tx (should have pk already published to controlled account)
    admin_account_id: crate::types::account_id::AccountId,

    #[interactive_clap(named_arg)]
    prepaid_gas: PrepaidGas,
}

// we need to get key or create a key that will be sent/derived
#[derive(Clone)]
pub struct SignMpcContext {
    smart_contract_addres: near_primitives::types::AccountId,
    admin_account_id: near_primitives::types::AccountId,
    tx_context: TransactionContext,
    mpc_tx_args: Vec<u8>,
    // add nounce and block height as we will know it at this point ig?
}

impl SignMpcContext {
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignMpc as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        // TODO: check the smart_contract_address and pk in persons account
        let mpc_tx_args = serde_json::to_vec(&serde_json::json!({"a": "a"}))?;
        Ok(SignMpcContext {
            smart_contract_addres: scope.smart_contract_address.clone().into(),
            admin_account_id: scope.admin_account_id.clone().into(),
            tx_context: previous_context,
            mpc_tx_args,
        })
    }
}

impl SignMpc {
    pub fn input_smart_contract_address(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the MPC contract AccountId?",
        )
    }

    pub fn input_admin_account_id(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the Admin AccountId?",
        )
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = SignMpcContext)]
#[interactive_clap(output_context = PrepaidGasContext)]
pub struct PrepaidGas {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter gas amount for contract call:
    gas: crate::common::NearGas,
    #[interactive_clap(named_arg)]
    /// Enter deposit for contract call:
    attached_deposit: Deposit,
}

#[derive(Clone)]
pub struct PrepaidGasContext {
    smart_contract_addres: near_primitives::types::AccountId,
    admin_account_id: near_primitives::types::AccountId,
    tx_context: TransactionContext,
    mpc_tx_args: Vec<u8>,
    gas: crate::common::NearGas,
}

impl PrepaidGasContext {
    pub fn from_previous_context(
        previous_context: SignMpcContext,
        scope: &<PrepaidGas as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(PrepaidGasContext {
            smart_contract_addres: previous_context.smart_contract_addres,
            admin_account_id: previous_context.admin_account_id,
            tx_context: previous_context.tx_context,
            mpc_tx_args: previous_context.mpc_tx_args,
            gas: scope.gas,
        })
    }
}

impl PrepaidGas {
    pub fn input_gas(
        _context: &SignMpcContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearGas>> {
        Ok(Some(
            CustomType::new("What is the gas limit for signing MPC (if unsure, keep 3 Tgas)?")
                .with_starting_input("3 Tgas")
                .with_validator(move |gas: &crate::common::NearGas| {
                    if gas > &near_gas::NearGas::from_tgas(300) {
                        Ok(inquire::validator::Validation::Invalid(
                            inquire::validator::ErrorMessage::Custom(
                                "You need to enter a value of no more than 300 TeraGas".to_string(),
                            ),
                        ))
                    } else {
                        Ok(inquire::validator::Validation::Valid)
                    }
                })
                .prompt()?,
        ))
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = PrepaidGasContext)]
#[interactive_clap(output_context = DepositContext)]
pub struct Deposit {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter deposit for MPC contract call:
    deposit: crate::types::near_token::NearToken,
    #[interactive_clap(subcommand)]
    transaction_signature_options: mpc_sign_with::MpcSignWith,
}

#[derive(Clone)]
pub struct DepositContext {
    smart_contract_addres: near_primitives::types::AccountId,
    admin_account_id: near_primitives::types::AccountId,
    tx_context: TransactionContext,
    mpc_tx_args: Vec<u8>,
    gas: crate::common::NearGas,
    deposit: crate::types::near_token::NearToken,
}

impl DepositContext {
    pub fn from_previous_context(
        previous_context: PrepaidGasContext,
        scope: &<Deposit as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(DepositContext {
            smart_contract_addres: previous_context.smart_contract_addres,
            admin_account_id: previous_context.admin_account_id,
            tx_context: previous_context.tx_context,
            mpc_tx_args: previous_context.mpc_tx_args,
            gas: previous_context.gas,
            deposit: scope.deposit,
        })
    }
}

impl Deposit {
    pub fn input_deposit(
        _context: &PrepaidGasContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::near_token::NearToken>> {
        Ok(Some(
            CustomType::new("Enter deposit for MPC contract call (if unsure, keep 0.1 NEAR):")
                .with_starting_input("0.1 NEAR")
                .prompt()?,
        ))
    }
}

impl From<DepositContext> for crate::commands::TransactionContext {
    fn from(item: DepositContext) -> Self {
        let new_prepopulated_transaction = crate::commands::PrepopulatedTransaction {
            signer_id: item.admin_account_id,
            receiver_id: item.smart_contract_addres,
            actions: vec![near_primitives::transaction::Action::FunctionCall(
                Box::new(near_primitives::transaction::FunctionCallAction {
                    method_name: "sign".to_string(),
                    args: item.mpc_tx_args,
                    gas: item.gas.as_gas(),
                    deposit: item.deposit.as_yoctonear(),
                }),
            )],
        };

        tracing::info!(
            "{}{}",
            "Unsigned MPC transaction",
            crate::common::indent_payload(&crate::common::print_unsigned_transaction(
                &new_prepopulated_transaction,
            ))
        );

        Self {
            global_context: item.tx_context.global_context,
            network_config: item.tx_context.network_config,
            prepopulated_transaction: new_prepopulated_transaction,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }
    }
}
