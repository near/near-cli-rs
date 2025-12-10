use std::str::FromStr;

use inquire::{CustomType, Select, Text};

#[derive(Debug, Clone)]
pub struct AccessKeyPermissionContext {
    pub global_context: crate::GlobalContext,
    pub signer_account_id: near_primitives::types::AccountId,
    pub receiver_account_id: near_primitives::types::AccountId,
    pub actions: Vec<near_primitives::transaction::Action>,
    pub access_key_permission: near_primitives::account::AccessKeyPermission,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::super::super::ConstructTransactionContext)]
#[interactive_clap(output_context = FullAccessTypeContext)]
pub struct FullAccessType {
    #[interactive_clap(subcommand)]
    access_key_mode: super::AccessKeyMode,
}

#[derive(Debug, Clone)]
pub struct FullAccessTypeContext(AccessKeyPermissionContext);

impl FullAccessTypeContext {
    pub fn from_previous_context(
        previous_context: super::super::super::super::ConstructTransactionContext,
        _scope: &<FullAccessType as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(AccessKeyPermissionContext {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            actions: previous_context.actions,
            access_key_permission: near_primitives::account::AccessKeyPermission::FullAccess,
        }))
    }
}

impl From<FullAccessTypeContext> for AccessKeyPermissionContext {
    fn from(item: FullAccessTypeContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::super::super::ConstructTransactionContext)]
#[interactive_clap(output_context = FunctionCallTypeContext)]
pub struct FunctionCallType {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    allowance: crate::types::near_allowance::NearAllowance,
    #[interactive_clap(long)]
    /// You chose to limit the access key to only sign transactions for a specific contract. Enter the contract account ID:
    contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    function_names: crate::types::vec_string::VecString,
    #[interactive_clap(subcommand)]
    access_key_mode: super::AccessKeyMode,
}

#[derive(Debug, Clone)]
pub struct FunctionCallTypeContext(AccessKeyPermissionContext);

impl FunctionCallTypeContext {
    pub fn from_previous_context(
        previous_context: super::super::super::super::ConstructTransactionContext,
        scope: &<FunctionCallType as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let access_key_permission = near_primitives::account::AccessKeyPermission::FunctionCall(
            near_primitives::account::FunctionCallPermission {
                allowance: scope.allowance.optional_near_token().map(Into::into),
                receiver_id: scope.contract_account_id.to_string(),
                method_names: scope.function_names.clone().into(),
            },
        );
        Ok(Self(AccessKeyPermissionContext {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            actions: previous_context.actions,
            access_key_permission,
        }))
    }
}

impl From<FunctionCallTypeContext> for AccessKeyPermissionContext {
    fn from(item: FunctionCallTypeContext) -> Self {
        item.0
    }
}

impl FunctionCallType {
    pub fn input_function_names(
        _context: &super::super::super::super::ConstructTransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::vec_string::VecString>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(
                to_string = "Yes, I want to input a list of function names that can be called when transaction is signed by this access key"
            )]
            Yes,
            #[strum(to_string = "No, I allow it to call any functions on the specified contract")]
            No,
        }
        let select_choose_input = Select::new(
            "Would you like the access key to be valid exclusively for calling specific functions on the contract?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let mut input_function_names =
                    Text::new("Enter a comma-separated list of function names that will be allowed to be called in a transaction signed by this access key:")
                        .prompt()?;
            if input_function_names.contains('\"') {
                input_function_names.clear()
            };
            if input_function_names.is_empty() {
                Ok(Some(crate::types::vec_string::VecString(vec![])))
            } else {
                Ok(Some(crate::types::vec_string::VecString::from_str(
                    &input_function_names,
                )?))
            }
        } else {
            Ok(Some(crate::types::vec_string::VecString(vec![])))
        }
    }

    pub fn input_allowance(
        _context: &super::super::super::super::ConstructTransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::near_allowance::NearAllowance>> {
        let allowance_near_balance: crate::types::near_allowance::NearAllowance =
            CustomType::new("Enter the allowance, a budget this access key can use to pay for transaction fees (example: 10NEAR or 0.5near or 10000yoctonear):")
                .with_starting_input("unlimited")
                .prompt()?;
        Ok(Some(allowance_near_balance))
    }
}
