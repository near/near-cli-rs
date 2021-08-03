use dialoguer::{console::Term, theme::ColorfulTheme, Input, Select};

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
    receiver_id: Option<near_primitives::types::AccountId>,
    #[clap(long)]
    method_names: Option<String>,
    #[clap(subcommand)]
    sign_option: Option<
        crate::commands::construct_transaction_command::sign_transaction::CliSignTransaction,
    >,
}

#[derive(Debug, Clone)]
pub struct FunctionCallType {
    pub allowance: Option<near_primitives::types::Balance>,
    pub receiver_id: near_primitives::types::AccountId,
    pub method_names: Vec<String>,
    pub sign_option:
        crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
}

impl CliFunctionCallType {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .sign_option
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
        if let Some(receiver_id) = &self.receiver_id {
            args.push_front(receiver_id.to_owned());
            args.push_front("--receiver-id".to_owned())
        };
        args
    }
}

impl From<FunctionCallType> for CliFunctionCallType {
    fn from(function_call_type: FunctionCallType) -> Self {
        Self{
            allowance: Some(crate::common::NearBalance::from_yoctonear(function_call_type.allowance.unwrap_or_default())),
            receiver_id: Some(function_call_type.receiver_id),
            method_names: Some(function_call_type.method_names.join(", ")),
            sign_option: Some(crate::commands::construct_transaction_command::sign_transaction::CliSignTransaction::from(function_call_type.sign_option)),
        }
    }
}

impl FunctionCallType {
    pub fn from(
        item: CliFunctionCallType,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: String,
    ) -> color_eyre::eyre::Result<Self> {
        let allowance: Option<near_primitives::types::Balance> = match item.allowance {
            Some(cli_allowance) => Some(cli_allowance.to_yoctonear()),
            None => FunctionCallType::input_allowance(),
        };
        let receiver_id: near_primitives::types::AccountId = match item.receiver_id {
            Some(cli_receiver_id) => near_primitives::types::AccountId::from(cli_receiver_id),
            None => FunctionCallType::input_receiver_id(),
        };
        let method_names: Vec<String> = match item.method_names {
            Some(cli_method_names) => {
                if cli_method_names.is_empty() {
                    vec![]
                } else {
                    cli_method_names
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect::<Vec<String>>()
                }
            }
            None => FunctionCallType::input_method_names(),
        };
        let sign_option = match item.sign_option {
            Some(cli_sign_transaction) => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::from(cli_sign_transaction, connection_config, sender_account_id)?,
            None => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::choose_sign_option(connection_config, sender_account_id)?,
        };
        Ok(Self {
            allowance,
            receiver_id,
            method_names,
            sign_option,
        })
    }
}

impl FunctionCallType {
    pub fn input_method_names() -> Vec<String> {
        println!();
        let choose_input = vec![
            "Yes, I want to input a list of method names that can be used",
            "No, I don't to input a list of method names that can be used",
        ];
        let select_choose_input = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Do You want to input a list of method names that can be used")
            .items(&choose_input)
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap();
        match select_choose_input {
            Some(0) => {
                let mut input_method_names: String = Input::new()
                    .with_prompt("Enter a comma-separated list of method names that will be allowed to be called in a transaction signed by this access key.")
                    .interact_text()
                    .unwrap();
                if input_method_names.contains("\"") {
                    input_method_names.clear()
                };
                if input_method_names.is_empty() {
                    vec![]
                } else {
                    input_method_names
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect::<Vec<String>>()
                }
            }
            Some(1) => vec![],
            _ => unreachable!("Error"),
        }
    }

    pub fn input_allowance() -> Option<near_primitives::types::Balance> {
        println!();
        let choose_input = vec![
            "Yes, I want to input allowance for receiver ID",
            "No, I don't to input allowance for receiver ID",
        ];
        let select_choose_input = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Do You want to input an allowance for receiver ID")
            .items(&choose_input)
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap();
        match select_choose_input {
            Some(0) => {
                let allowance_near_balance: crate::common::NearBalance = Input::new()
                    .with_prompt("Enter an allowance which is a balance limit to use by this access key to pay for function call gas and transaction fees.")
                    .interact_text()
                    .unwrap();
                Some(allowance_near_balance.to_yoctonear())
            }
            Some(1) => None,
            _ => unreachable!("Error"),
        }
    }

    pub fn input_receiver_id() -> near_primitives::types::AccountId {
        println!();
        Input::new()
            .with_prompt("Enter a receiver to use by this access key to pay for function call gas and transaction fees.")
            .interact_text()
            .unwrap()
    }

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
                    allowance: self.allowance.clone(),
                    receiver_id: self.receiver_id.clone(),
                    method_names: self.method_names.clone(),
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
                )
                .await;
            }
            None => {}
        };
        Ok(())
    }
}
