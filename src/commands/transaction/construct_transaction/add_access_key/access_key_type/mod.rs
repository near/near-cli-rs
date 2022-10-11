use dialoguer::{console::Term, theme::ColorfulTheme, Input, Select};
use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct FullAccessType {
    #[interactive_clap(subcommand)]
    access_key_mode: super::AccessKeyMode,
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
#[interactive_clap(context = crate::GlobalContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct FunctionCallType {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    allowance: Option<crate::common::NearBalance>,
    #[interactive_clap(long)]
    ///Enter a receiver to use by this access key to pay for function call gas and transaction fees.
    receiver_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    method_names: crate::types::vec_string::VecString,
    #[interactive_clap(subcommand)]
    access_key_mode: super::AccessKeyMode,
}

impl FunctionCallType {
    pub fn from_cli(
        optional_clap_variant: Option<<FunctionCallType as interactive_clap::ToCli>::CliVariant>,
        context: crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<Self>> {
        let allowance: Option<crate::common::NearBalance> = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.allowance)
        {
            Some(cli_allowance) => Some(cli_allowance),
            None => FunctionCallType::input_allowance()?,
        };
        let receiver_account_id = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.receiver_account_id)
        {
            Some(cli_receiver_account_id) => cli_receiver_account_id,
            None => Self::input_receiver_account_id(&context)?,
        };
        let method_names: crate::types::vec_string::VecString = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.method_names)
        {
            Some(cli_method_names) => {
                if cli_method_names.0.is_empty() {
                    crate::types::vec_string::VecString(vec![])
                } else {
                    cli_method_names
                }
            }
            None => FunctionCallType::input_method_names()?,
        };
        let optional_access_key_mode =
            match optional_clap_variant.and_then(|clap_variant| clap_variant.access_key_mode) {
                Some(cli_access_key_mode) => {
                    super::AccessKeyMode::from_cli(Some(cli_access_key_mode), context)?
                }
                None => super::AccessKeyMode::choose_variant(context)?,
            };
        let access_key_mode = if let Some(access_key_mode) = optional_access_key_mode {
            access_key_mode
        } else {
            return Ok(None);
        };
        Ok(Some(Self {
            allowance,
            receiver_account_id,
            method_names,
            access_key_mode,
        }))
    }
}

impl FunctionCallType {
    pub fn input_method_names() -> color_eyre::eyre::Result<crate::types::vec_string::VecString> {
        println!();
        let choose_input = vec![
            "Yes, I want to input a list of method names that can be used",
            "No, I don't to input a list of method names that can be used",
        ];
        let select_choose_input = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Do You want to input a list of method names that can be used")
            .items(&choose_input)
            .default(0)
            .interact_on_opt(&Term::stderr())?;
        match select_choose_input {
            Some(0) => {
                let mut input_method_names: String = Input::new()
                    .with_prompt("Enter a comma-separated list of method names that will be allowed to be called in a transaction signed by this access key.")
                    .interact_text()
                    ?;
                if input_method_names.contains('\"') {
                    input_method_names.clear()
                };
                if input_method_names.is_empty() {
                    Ok(crate::types::vec_string::VecString(vec![]))
                } else {
                    crate::types::vec_string::VecString::from_str(&input_method_names)
                }
            }
            Some(1) => Ok(crate::types::vec_string::VecString(vec![])),
            _ => unreachable!("Error"),
        }
    }

    pub fn input_allowance() -> color_eyre::eyre::Result<Option<crate::common::NearBalance>> {
        println!();
        let choose_input = vec![
            "Yes, I want to input allowance for receiver ID",
            "No, I don't to input allowance for receiver ID",
        ];
        let select_choose_input = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Do You want to input an allowance for receiver ID")
            .items(&choose_input)
            .default(0)
            .interact_on_opt(&Term::stderr())?;
        match select_choose_input {
            Some(0) => {
                let allowance_near_balance: crate::common::NearBalance = Input::new()
                    .with_prompt("Enter an allowance which is a balance limit to use by this access key to pay for function call gas and transaction fees. (example: 10NEAR or 0.5near or 10000yoctonear)")
                    .interact_text()
                    ?;
                Ok(Some(allowance_near_balance))
            }
            Some(1) => Ok(None),
            _ => unreachable!("Error"),
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
