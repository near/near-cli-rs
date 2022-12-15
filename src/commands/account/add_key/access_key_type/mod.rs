use inquire::{CustomType, Select, Text};
use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct FullAccessType {
    #[interactive_clap(subcommand)]
    pub access_key_mode: super::AccessKeyMode,
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

impl interactive_clap::FromCli for FunctionCallType {
    type FromCliContext = crate::GlobalContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<FunctionCallType as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> Result<Option<Self>, Self::FromCliError>
    where
        Self: Sized + interactive_clap::ToCli,
    {
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

    pub fn input_allowance() -> color_eyre::eyre::Result<Option<crate::common::NearBalance>> {
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
