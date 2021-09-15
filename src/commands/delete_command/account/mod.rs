use dialoguer::Input;

pub mod operation_mode;
mod sender;

/// удаление аккаунта
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliDeleteAccountAction {
    beneficiary_id: Option<near_primitives::types::AccountId>,
    #[clap(subcommand)]
    sign_option: Option<
        crate::commands::construct_transaction_command::sign_transaction::CliSignTransaction,
    >,
}

#[derive(Debug, Clone)]
pub struct DeleteAccountAction {
    pub beneficiary_id: near_primitives::types::AccountId,
    pub sign_option:
        crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
}

impl CliDeleteAccountAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .sign_option
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(beneficiary_id) = &self.beneficiary_id {
            args.push_front(beneficiary_id.to_string());
        }
        args
    }
}

impl From<DeleteAccountAction> for CliDeleteAccountAction {
    fn from(delete_account_action: DeleteAccountAction) -> Self {
        Self {
            beneficiary_id: Some(delete_account_action.beneficiary_id),
            sign_option: Some(delete_account_action.sign_option.into()),
        }
    }
}

impl DeleteAccountAction {
    pub fn from(
        item: CliDeleteAccountAction,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        let beneficiary_id: near_primitives::types::AccountId = match item.beneficiary_id {
            Some(cli_account_id) => match &connection_config {
                Some(network_connection_config) => match crate::common::get_account_state(
                    network_connection_config,
                    cli_account_id.clone(),
                )? {
                    Some(_) => cli_account_id,
                    None => {
                        println!("Account <{}> doesn't exist", cli_account_id);
                        DeleteAccountAction::input_beneficiary_id(connection_config.clone())?
                    }
                },
                None => cli_account_id,
            },
            None => DeleteAccountAction::input_beneficiary_id(connection_config.clone())?,
        };
        let sign_option = match item.sign_option {
            Some(cli_sign_transaction) => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::from(cli_sign_transaction, connection_config, sender_account_id)?,
            None => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::choose_sign_option(connection_config, sender_account_id)?,
        };
        Ok(Self {
            beneficiary_id,
            sign_option,
        })
    }
}

impl DeleteAccountAction {
    pub fn input_beneficiary_id(
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<near_primitives::types::AccountId> {
        loop {
            let account_id: near_primitives::types::AccountId = Input::new()
                .with_prompt("Enter the beneficiary ID to delete this account ID")
                .interact_text()
                .unwrap();
            if let Some(connection_config) = &connection_config {
                if let Some(_) =
                    crate::common::get_account_state(connection_config, account_id.clone())?
                {
                    break Ok(account_id);
                } else {
                    println!("Account <{}> doesn't exist", account_id);
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
    ) -> crate::CliResult {
        let beneficiary_id: near_primitives::types::AccountId = self.beneficiary_id.clone();
        let action = near_primitives::transaction::Action::DeleteAccount(
            near_primitives::transaction::DeleteAccountAction { beneficiary_id },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        match self
            .sign_option
            .process(unsigned_transaction, network_connection_config.clone())
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
