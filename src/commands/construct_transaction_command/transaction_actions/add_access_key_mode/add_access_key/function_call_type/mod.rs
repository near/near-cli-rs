use async_recursion::async_recursion;
use dialoguer::{console::Term, theme::ColorfulTheme, Input, Select};
use std::vec;

/// данные для определения ключа с function call
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliFunctionCallType {
    #[clap(long)]
    allowance: Option<crate::common::NearBalance>,
    #[clap(long)]
    pub receiver_account_id: Option<crate::types::account_id::AccountId>,
    #[clap(long)]
    method_names: Option<String>,
    #[clap(subcommand)]
    next_action: Option<super::super::super::CliSkipNextAction>,
}

#[derive(Debug, Clone)]
pub struct FunctionCallType {
    pub allowance: Option<crate::common::NearBalance>,
    pub receiver_account_id: crate::types::account_id::AccountId,
    pub method_names: Vec<String>,
    pub next_action: Box<super::super::super::NextAction>,
}

impl interactive_clap::ToCli for FunctionCallType {
    type CliVariant = CliFunctionCallType;
}

impl CliFunctionCallType {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .next_action
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(method_names) = &self.method_names {
            args.push_front(method_names.to_string());
            args.push_front("--method-names".to_owned())
        };
        if let Some(allowance) = &self.allowance {
            args.push_front(allowance.to_string());
            args.push_front("--allowance".to_owned())
        };
        if let Some(receiver_id) = &self.receiver_account_id {
            args.push_front(receiver_id.to_string());
            args.push_front("--receiver-account-id".to_owned())
        };
        args
    }
}

impl From<FunctionCallType> for CliFunctionCallType {
    fn from(function_call_type: FunctionCallType) -> Self {
        Self {
            allowance: function_call_type.allowance,
            receiver_account_id: Some(function_call_type.receiver_account_id),
            method_names: Some(function_call_type.method_names.join(", ")),
            next_action: Some(super::super::super::CliSkipNextAction::Skip(
                super::super::super::CliSkipAction { sign_option: None },
            )),
        }
    }
}

impl FunctionCallType {
    pub fn from_cli(
        optional_clap_variant: Option<CliFunctionCallType>,
        context: crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<Self> {
        let connection_config = context.connection_config.clone();
        let allowance: Option<crate::common::NearBalance> = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.allowance)
        {
            Some(cli_allowance) => Some(cli_allowance),
            None => FunctionCallType::input_allowance(&context)?,
        };
        let receiver_account_id = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.receiver_account_id)
        {
            Some(receiver_account_id) => match &connection_config {
                Some(network_connection_config) => match crate::common::get_account_state(
                    &network_connection_config,
                    receiver_account_id.clone().into(),
                )? {
                    Some(_) => receiver_account_id,
                    None => {
                        println!("Account <{}> doesn't exist", receiver_account_id);
                        Self::input_receiver_account_id(&context)?
                    }
                },
                None => receiver_account_id,
            },
            None => Self::input_receiver_account_id(&context)?,
        };
        let method_names: Vec<String> = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.method_names)
        {
            Some(cli_method_names) => {
                if cli_method_names.is_empty() {
                    vec![]
                } else {
                    cli_method_names
                        .split(',')
                        .map(String::from)
                        .collect::<Vec<String>>()
                }
            }
            None => FunctionCallType::input_method_names(&context)?,
        };
        let skip_next_action: super::super::super::NextAction = match optional_clap_variant
            .and_then(|clap_variant| clap_variant.next_action)
        {
            Some(cli_skip_action) => super::super::super::NextAction::from_cli_skip_next_action(
                cli_skip_action,
                context,
            )?,
            None => super::super::super::NextAction::choose_variant(context)?,
        };
        Ok(Self {
            allowance,
            receiver_account_id,
            method_names,
            next_action: Box::new(skip_next_action),
        })
    }
}

impl FunctionCallType {
    pub fn input_method_names(
        _context: &crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<Vec<String>> {
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
                if input_method_names.contains("\"") {
                    input_method_names.clear()
                };
                if input_method_names.is_empty() {
                    Ok(vec![])
                } else {
                    Ok(input_method_names
                        .split(',')
                        .map(String::from)
                        .collect::<Vec<String>>())
                }
            }
            Some(1) => Ok(vec![]),
            _ => unreachable!("Error"),
        }
    }

    pub fn input_allowance(
        _context: &crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearBalance>> {
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
                    .with_prompt("Enter an allowance which is a balance limit to use by this access key to pay for function call gas and transaction fees.")
                    .interact_text()
                    ?;
                Ok(Some(allowance_near_balance))
            }
            Some(1) => Ok(None),
            _ => unreachable!("Error"),
        }
    }

    pub fn input_receiver_account_id(
        context: &crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        let connection_config = context.connection_config.clone();
        loop {
            let account_id: crate::types::account_id::AccountId = Input::new()
                .with_prompt("Enter a receiver to use by this access key to pay for function call gas and transaction fees.")
                .interact_text()
                ?;
            if let Some(connection_config) = &connection_config {
                if let Some(_) =
                    crate::common::get_account_state(&connection_config, account_id.clone().into())?
                {
                    break Ok(account_id);
                } else {
                    if !crate::common::is_64_len_hex(&account_id) {
                        println!("Account <{}> doesn't exist", account_id.to_string());
                    } else {
                        break Ok(account_id);
                    }
                }
            } else {
                break Ok(account_id);
            }
        }
    }

    #[async_recursion(?Send)]
    pub async fn process(
        self,
        nonce: near_primitives::types::Nonce,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
        public_key: near_crypto::PublicKey,
    ) -> crate::CliResult {
        let access_key: near_primitives::account::AccessKey = near_primitives::account::AccessKey {
            nonce,
            permission: near_primitives::account::AccessKeyPermission::FunctionCall(
                near_primitives::account::FunctionCallPermission {
                    allowance: {
                        match self.allowance.clone() {
                            Some(allowance) => Some(allowance.to_yoctonear()),
                            None => None,
                        }
                    },
                    receiver_id: self.receiver_account_id.to_string().clone(),
                    method_names: self.method_names.clone(),
                },
            ),
        };
        let action = near_primitives::transaction::Action::AddKey(
            near_primitives::transaction::AddKeyAction {
                public_key,
                access_key,
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        match *self.next_action {
            super::super::super::NextAction::AddAction(select_action) => {
                select_action
                    .process(unsigned_transaction, network_connection_config)
                    .await
            }
            super::super::super::NextAction::Skip(skip_action) => {
                skip_action
                    .process(unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}
