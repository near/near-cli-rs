use dialoguer::Input;

pub mod operation_mode;
mod sender;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SenderContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct DeleteAccountAction {
    pub beneficiary_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    pub sign_option:
        crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
}

impl DeleteAccountAction {
    pub fn from_cli(
        optional_clap_variant: Option<CliDeleteAccountAction>,
        context: crate::common::SenderContext,
    ) -> color_eyre::eyre::Result<Self> {
        let connection_config = context.connection_config.clone();
        let beneficiary_id = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.beneficiary_id)
        {
            Some(beneficiary_id) => match &connection_config {
                Some(network_connection_config) => match crate::common::get_account_state(
                    &network_connection_config,
                    beneficiary_id.clone().into(),
                )? {
                    Some(_) => beneficiary_id,
                    None => {
                        println!("Account <{}> doesn't exist", beneficiary_id);
                        Self::input_beneficiary_id(&context)?
                    }
                },
                None => beneficiary_id,
            },
            None => Self::input_beneficiary_id(&context)?,
        };
        let sign_option = match optional_clap_variant.and_then(|clap_variant| clap_variant.sign_option) {
            Some(cli_sign_transaction) => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::from_cli(Some(cli_sign_transaction), context)?,
            None => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::choose_variant(context)?,
        };
        Ok(Self {
            beneficiary_id,
            sign_option,
        })
    }
}

impl DeleteAccountAction {
    pub fn input_beneficiary_id(
        context: &crate::common::SenderContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        let connection_config = context.connection_config.clone();
        loop {
            let account_id: crate::types::account_id::AccountId = Input::new()
                .with_prompt("Enter the beneficiary ID to delete this account ID")
                .interact_text()
                .unwrap();
            if let Some(connection_config) = &connection_config {
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
