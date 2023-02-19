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
#[interactive_clap(output_context = AccessTypeContext)]
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

impl FullAccessType {
    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.access_key_mode
            .process(
                config,
                prepopulated_unsigned_transaction,
                near_primitives::account::AccessKeyPermission::FullAccess,
            )
            .await
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::AddKeyCommandContext)]
#[interactive_clap(output_context = AccessTypeContext)]
pub struct FunctionCallType {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    allowance: Option<crate::common::NearBalance>,
    #[interactive_clap(long)]
    ///Enter a receiver to use by this access key to pay for function call gas and transaction fees.
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
                    allowance: item
                        .allowance
                        // .clone()
                        .map(|allowance| allowance.to_yoctonear()),
                    receiver_id: item.receiver_account_id.to_string(),
                    method_names: item.method_names.into(),
                },
            ),
        }
    }
}

impl FunctionCallType {
    pub fn input_method_names(
        _context: &super::AddKeyCommandContext,
    ) -> color_eyre::eyre::Result<crate::types::vec_string::VecString> {
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, I want to input a list of method names that can be used")]
            Yes,
            #[strum(
                to_string = "No, I don't want to input a list of method names that can be used"
            )]
            No,
        }

        println!();
        let select_choose_input = Select::new(
            "Do You want to input a list of method names that can be used",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let mut input_method_names = Text::new("Enter a comma-separated list of method names that will be allowed to be called in a transaction signed by this access key.")
                    .prompt()?;
            if input_method_names.contains('\"') {
                input_method_names.clear()
            };
            if input_method_names.is_empty() {
                Ok(crate::types::vec_string::VecString(vec![]))
            } else {
                crate::types::vec_string::VecString::from_str(&input_method_names)
            }
        } else {
            Ok(crate::types::vec_string::VecString(vec![]))
        }
    }

    pub fn input_allowance(
        _context: &super::AddKeyCommandContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearBalance>> {
        println!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, I want to input allowance for receiver ID")]
            Yes,
            #[strum(to_string = "No, I don't want to input allowance for receiver ID")]
            No,
        }
        let select_choose_input = Select::new(
            "Do You want to input an allowance for receiver ID",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let allowance_near_balance: crate::common::NearBalance =
                    CustomType::new("Enter an allowance which is a balance limit to use by this access key to pay for function call gas and transaction fees. (example: 10NEAR or 0.5near or 10000yoctonear)")
                    .prompt()?;
            Ok(Some(allowance_near_balance))
        } else {
            Ok(None)
        }
    }

    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        let permission = near_primitives::account::AccessKeyPermission::FunctionCall(
            near_primitives::account::FunctionCallPermission {
                allowance: self
                    .allowance
                    .clone()
                    .map(|allowance| allowance.to_yoctonear()),
                receiver_id: self.receiver_account_id.to_string(),
                method_names: self.method_names.clone().into(),
            },
        );
        self.access_key_mode
            .process(config, prepopulated_unsigned_transaction, permission)
            .await
    }
}
