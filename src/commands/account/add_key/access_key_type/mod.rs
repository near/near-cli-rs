use std::str::FromStr;

use inquire::{CustomType, Select, Text};

#[derive(Debug, Clone)]
pub struct AccessTypeContext {
    pub config: crate::config::Config,
    pub signer_account_id: near_primitives::types::AccountId,
    pub permission: near_primitives::account::AccessKeyPermission,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::AddKeyCommandContext)]
#[interactive_clap(output_context = FullAccessTypeContext)]
pub struct FullAccessType {
    #[interactive_clap(subcommand)]
    pub access_key_mode: super::AccessKeyMode,
}

#[derive(Debug, Clone)]
pub struct FullAccessTypeContext {
    config: crate::config::Config,
    signer_account_id: near_primitives::types::AccountId,
    permission: near_primitives::account::AccessKeyPermission,
}

impl FullAccessTypeContext {
    pub fn from_previous_context(
        previous_context: super::AddKeyCommandContext,
        _scope: &<FullAccessType as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            signer_account_id: previous_context.owner_account_id.into(),
            permission: near_primitives::account::AccessKeyPermission::FullAccess,
        })
    }
}

impl From<FullAccessTypeContext> for AccessTypeContext {
    fn from(item: FullAccessTypeContext) -> Self {
        Self {
            config: item.config,
            signer_account_id: item.signer_account_id,
            permission: item.permission,
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::AddKeyCommandContext)]
#[interactive_clap(output_context = AccessTypeContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct FunctionCallType {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    allowance: Option<crate::common::NearBalance>,
    #[interactive_clap(long)]
    /// Enter a receiver to use by this access key to pay for function call gas and transaction fees
    receiver_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    method_names: crate::types::vec_string::VecString,
    #[interactive_clap(subcommand)]
    access_key_mode: super::AccessKeyMode,
}

#[derive(Debug, Clone)]
pub struct FunctionCallTypeContext {
    config: crate::config::Config,
    signer_account_id: near_primitives::types::AccountId,
    allowance: Option<crate::common::NearBalance>,
    receiver_account_id: crate::types::account_id::AccountId,
    method_names: crate::types::vec_string::VecString,
}

impl FunctionCallTypeContext {
    pub fn from_previous_context(
        previous_context: super::AddKeyCommandContext,
        scope: &<FunctionCallType as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            signer_account_id: previous_context.owner_account_id.into(),
            allowance: scope.allowance.clone(),
            receiver_account_id: scope.receiver_account_id.clone(),
            method_names: scope.method_names.clone(),
        })
    }
}

impl From<FunctionCallTypeContext> for AccessTypeContext {
    fn from(item: FunctionCallTypeContext) -> Self {
        Self {
            config: item.config,
            signer_account_id: item.signer_account_id,
            permission: near_primitives::account::AccessKeyPermission::FunctionCall(
                near_primitives::account::FunctionCallPermission {
                    allowance: item.allowance.map(|allowance| allowance.to_yoctonear()),
                    receiver_id: item.receiver_account_id.to_string(),
                    method_names: item.method_names.into(),
                },
            ),
        }
    }
}

impl interactive_clap::FromCli for FunctionCallType {
    type FromCliContext = super::AddKeyCommandContext;
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
            clap_variant.allowance = match Self::input_allowance(&context) {
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
            clap_variant.method_names = match Self::input_method_names(&context) {
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
        let new_context =
            match FunctionCallTypeContext::from_previous_context(context, &new_context_scope) {
                Ok(new_context) => new_context,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        let output_context = AccessTypeContext::from(new_context);

        match super::AccessKeyMode::from_cli(clap_variant.access_key_mode.take(), output_context) {
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
        _context: &super::AddKeyCommandContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::vec_string::VecString>> {
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, I want to input a list of method names that can be used")]
            Yes,
            #[strum(
                to_string = "No, I don't want to input a list of method names that can be used"
            )]
            No,
        }

        eprintln!();
        let select_choose_input = Select::new(
            "Do You want to input a list of method names that can be used?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let mut input_method_names = Text::new("Enter a comma-separated list of method names that will be allowed to be called in a transaction signed by this access key:")
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

    pub fn input_allowance(
        _context: &super::AddKeyCommandContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearBalance>> {
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
            let allowance_near_balance: crate::common::NearBalance =
                    CustomType::new("Enter an allowance which is a balance limit to use by this access key to pay for function call gas and transaction fees (example: 10NEAR or 0.5near or 10000yoctonear):")
                    .prompt()?;
            Ok(Some(allowance_near_balance))
        } else {
            Ok(None)
        }
    }
}
