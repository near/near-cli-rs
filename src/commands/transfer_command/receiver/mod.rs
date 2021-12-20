use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SenderContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct Receiver {
    pub receiver_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    pub transfer: super::transfer_near_tokens_type::Transfer,
}

impl interactive_clap::ToCli for crate::types::account_id::AccountId {
    type CliVariant = crate::types::account_id::AccountId;
}

impl Receiver {
    pub fn from_cli(
        optional_clap_variant: Option<CliReceiver>,
        context: crate::common::SenderContext,
    ) -> color_eyre::eyre::Result<Self> {
        let connection_config = context.connection_config.clone();
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
        let transfer = super::transfer_near_tokens_type::Transfer::from_cli(
            optional_clap_variant.and_then(|clap_variant| clap_variant.transfer),
            context,
        )?;

        Ok(Self {
            receiver_account_id,
            transfer,
        })
    }
}

impl Receiver {
    pub fn input_receiver_account_id(
        context: &crate::common::SenderContext,
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
