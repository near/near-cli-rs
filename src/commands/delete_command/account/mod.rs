use dialoguer::Input;

pub mod operation_mode;
mod sender;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SignerContext)]
pub struct DeleteAccountAction {
    #[interactive_clap(skip_default_from_cli)]
    pub beneficiary_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    pub sign_option:
        crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
}

impl DeleteAccountAction {
    fn from_cli_beneficiary_id(
        optional_cli_sender_account_id: Option<crate::types::account_id::AccountId>,
        context: &crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        match optional_cli_sender_account_id {
            Some(cli_beneficiary_id) => match &context.connection_config {
                Some(network_connection_config) => match crate::common::get_account_state(
                    &network_connection_config,
                    cli_beneficiary_id.clone().into(),
                )? {
                    Some(_) => Ok(cli_beneficiary_id),
                    None => {
                        println!("Account <{}> doesn't exist", cli_beneficiary_id);
                        Self::input_beneficiary_id(&context)
                    }
                },
                None => Ok(cli_beneficiary_id),
            },
            None => Self::input_beneficiary_id(&context),
        }
    }

    pub fn input_beneficiary_id(
        context: &crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        loop {
            let account_id: crate::types::account_id::AccountId = Input::new()
                .with_prompt("Enter the beneficiary ID to delete this account ID")
                .interact_text()?;
            if let Some(connection_config) = &context.connection_config {
                if let Some(_) =
                    crate::common::get_account_state(&connection_config, account_id.clone().into())?
                {
                    break Ok(account_id);
                } else {
                    println!("Account <{}> doesn't exist", account_id.to_string());
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
        let beneficiary_id: near_primitives::types::AccountId = self.beneficiary_id.clone().into();
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
