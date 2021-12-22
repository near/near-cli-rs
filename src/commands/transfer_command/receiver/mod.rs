use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SignerContext)]
pub struct Receiver {
    #[interactive_clap(skip_default_from_cli)]
    pub receiver_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    pub transfer: super::transfer_near_tokens_type::Transfer,
}

impl Receiver {
    fn from_cli_receiver_account_id(
        optional_cli_sender_account_id: Option<crate::types::account_id::AccountId>,
        context: &crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        match optional_cli_sender_account_id {
            Some(cli_receiver_account_id) => match &context.connection_config {
                Some(network_connection_config) => match crate::common::get_account_state(
                    &network_connection_config,
                    cli_receiver_account_id.clone().into(),
                )? {
                    Some(_) => Ok(cli_receiver_account_id),
                    None => {
                        println!("Account <{}> doesn't exist", cli_receiver_account_id);
                        Self::input_receiver_account_id(&context)
                    }
                },
                None => Ok(cli_receiver_account_id),
            },
            None => Self::input_receiver_account_id(&context),
        }
    }

    pub fn input_receiver_account_id(
        context: &crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        let connection_config = context.connection_config.clone();
        loop {
            let account_id: crate::types::account_id::AccountId = Input::new()
                .with_prompt("What is the account ID of the receiver?")
                .interact_text()
                .unwrap();
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
    ) -> crate::CliResult {
        let unsigned_transaction = near_primitives::transaction::Transaction {
            receiver_id: self.receiver_account_id.clone().into(),
            ..prepopulated_unsigned_transaction
        };
        self.transfer
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
