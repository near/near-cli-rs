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
#[interactive_clap(skip_default_from_cli)]
pub struct FunctionCallType {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    allowance: Option<near_token::NearToken>,
    #[interactive_clap(long)]
    /// Enter a receiver to use by this access key to pay for function call gas and transaction fees:
    receiver_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    method_names: crate::types::vec_string::VecString,
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
                allowance: scope
                    .allowance
                    .clone()
                    .map(|allowance| allowance.as_yoctonear()),
                receiver_id: scope.receiver_account_id.to_string(),
                method_names: scope.method_names.clone().into(),
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

impl interactive_clap::FromCli for FunctionCallType {
    type FromCliContext = super::super::super::super::ConstructTransactionContext;
    type FromCliError = color_eyre::eyre::Error;
    fn from_cli(
        optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();

        if clap_variant.allowance.is_none() {
            clap_variant.allowance = match Self::input_allowance() {
                Ok(optional_allowance) => optional_allowance,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let allowance = clap_variant.allowance.clone();
        if clap_variant.receiver_account_id.is_none() {
            clap_variant.receiver_account_id = match Self::input_receiver_account_id(&context) {
                Ok(Some(first_receiver_account_id)) => Some(first_receiver_account_id),
                Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let receiver_account_id = clap_variant
            .receiver_account_id
            .clone()
            .expect("Unexpected error");
        if clap_variant.method_names.is_none() {
            clap_variant.method_names = match Self::input_method_names() {
                Ok(Some(first_method_names)) => Some(first_method_names),
                Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let method_names = clap_variant.method_names.clone().expect("Unexpected error");

        let new_context_scope = InteractiveClapContextScopeForFunctionCallType {
            allowance,
            receiver_account_id,
            method_names,
        };
        let output_context =
            match FunctionCallTypeContext::from_previous_context(context, &new_context_scope) {
                Ok(new_context) => new_context,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };

        match super::AccessKeyMode::from_cli(
            clap_variant.access_key_mode.take(),
            output_context.into(),
        ) {
            interactive_clap::ResultFromCli::Ok(cli_access_key_mode) => {
                clap_variant.access_key_mode = Some(cli_access_key_mode);
                interactive_clap::ResultFromCli::Ok(clap_variant)
            }
            interactive_clap::ResultFromCli::Cancel(optional_cli_access_key_mode) => {
                clap_variant.access_key_mode = optional_cli_access_key_mode;
                interactive_clap::ResultFromCli::Cancel(Some(clap_variant))
            }
            interactive_clap::ResultFromCli::Back => interactive_clap::ResultFromCli::Back,
            interactive_clap::ResultFromCli::Err(optional_cli_access_key_mode, err) => {
                clap_variant.access_key_mode = optional_cli_access_key_mode;
                interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
            }
        }
    }
}

impl FunctionCallType {
    pub fn input_method_names(
    ) -> color_eyre::eyre::Result<Option<crate::types::vec_string::VecString>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, I want to input a list of method names that can be used")]
            Yes,
            #[strum(
                to_string = "No, I don't want to input a list of method names that can be used"
            )]
            No,
        }
        let select_choose_input = Select::new(
            "Do You want to input a list of method names that can be used?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let mut input_method_names =
                    Text::new("Enter a comma-separated list of method names that will be allowed to be called in a transaction signed by this access key:")
                        .prompt()?;
            if input_method_names.contains('\"') {
                input_method_names.clear()
            };
            if input_method_names.is_empty() {
                Ok(Some(crate::types::vec_string::VecString(vec![])))
            } else {
                Ok(Some(crate::types::vec_string::VecString::from_str(
                    &input_method_names,
                )?))
            }
        } else {
            Ok(Some(crate::types::vec_string::VecString(vec![])))
        }
    }

    pub fn input_allowance() -> color_eyre::eyre::Result<Option<near_token::NearToken>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, I want to input allowance for receiver ID")]
            Yes,
            #[strum(to_string = "No, I don't want to input allowance for receiver ID")]
            No,
        }
        let select_choose_input = Select::new(
            "Do You want to input an allowance for receiver ID?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let allowance_near_balance: near_token::NearToken =
                    CustomType::new("Enter an allowance which is a balance limit to use by this access key to pay for function call gas and transaction fees (example: 10NEAR or 0.5near or 10000yoctonear):")
                        .prompt()?;
            Ok(Some(allowance_near_balance))
        } else {
            Ok(None)
        }
    }
}
