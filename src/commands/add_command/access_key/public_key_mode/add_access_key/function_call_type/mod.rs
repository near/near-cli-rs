use std::str::FromStr;

use dialoguer::{console::Term, theme::ColorfulTheme, Input, Select};

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SignerContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct FunctionCallType {
    #[interactive_clap(long)]
    pub allowance: Option<crate::common::NearBalance>,
    #[interactive_clap(long)]
    pub receiver_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(long)]
    pub method_names: crate::types::vec_string::VecString,
    #[interactive_clap(subcommand)]
    pub sign_option:
        crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
}

impl interactive_clap::ToCli for crate::types::vec_string::VecString {
    type CliVariant = crate::types::vec_string::VecString;
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
            None => FunctionCallType::input_method_names(&context)?,
        };
        let sign_option = match optional_clap_variant.and_then(|clap_variant| clap_variant.sign_option) {
            Some(cli_sign_transaction) => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::from_cli(Some(cli_sign_transaction), context)?,
            None => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::choose_variant(context)?,
        };
        Ok(Self {
            allowance,
            receiver_account_id,
            method_names,
            sign_option,
        })
    }
}

impl FunctionCallType {
    pub fn input_method_names(
        _context: &crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<crate::types::vec_string::VecString> {
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
                    Ok(crate::types::vec_string::VecString(vec![]))
                } else {
                    crate::types::vec_string::VecString::from_str(&input_method_names)
                }
            }
            Some(1) => Ok(crate::types::vec_string::VecString(vec![])),
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

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
        public_key: near_crypto::PublicKey,
    ) -> crate::CliResult {
        let access_key: near_primitives::account::AccessKey = near_primitives::account::AccessKey {
            nonce: 0,
            permission: near_primitives::account::AccessKeyPermission::FunctionCall(
                near_primitives::account::FunctionCallPermission {
                    allowance: {
                        match self.allowance.clone() {
                            Some(allowance) => Some(allowance.to_yoctonear()),
                            None => None,
                        }
                    },
                    receiver_id: self.receiver_account_id.to_string().clone(),
                    method_names: self.method_names.clone().into(),
                },
            ),
        };
        let action = near_primitives::transaction::Action::AddKey(
            near_primitives::transaction::AddKeyAction {
                public_key: public_key.clone(),
                access_key,
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        match self
            .sign_option
            .process(
                unsigned_transaction.clone(),
                network_connection_config.clone(),
            )
            .await?
        {
            Some(transaction_info) => {
                crate::common::print_transaction_status(
                    transaction_info,
                    network_connection_config,
                );
            }
            None => {}
        };
        Ok(())
    }
}
