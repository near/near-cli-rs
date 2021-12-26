use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::operation_mode::AddSubAccountCommandNetworkContext)]
#[interactive_clap(output_context = crate::common::SignerContext)]
pub struct Sender {
    #[interactive_clap(skip_default_from_cli)]
    pub owner_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    ///Specify a sub-account
    pub sub_account: super::receiver::SubAccount,
}

struct SenderContext {
    connection_config: Option<crate::common::ConnectionConfig>,
    sender_account_id: crate::types::account_id::AccountId,
}

impl SenderContext {
    pub fn from_previous_context(
        previous_context: super::operation_mode::AddSubAccountCommandNetworkContext,
        scope: &<Sender as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Self {
        Self {
            connection_config: previous_context.connection_config.clone(),
            sender_account_id: scope.owner_account_id.clone(),
        }
    }
}

impl From<SenderContext> for crate::common::SignerContext {
    fn from(item: SenderContext) -> Self {
        Self {
            connection_config: item.connection_config,
            signer_account_id: item.sender_account_id,
        }
    }
}

impl Sender {
    fn from_cli_owner_account_id(
        optional_cli_owner_account_id: Option<crate::types::account_id::AccountId>,
        context: &super::operation_mode::AddSubAccountCommandNetworkContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        match optional_cli_owner_account_id {
            Some(cli_owner_account_id) => match &context.connection_config {
                Some(network_connection_config) => match crate::common::get_account_state(
                    &network_connection_config,
                    cli_owner_account_id.clone().into(),
                )? {
                    Some(_) => Ok(cli_owner_account_id),
                    None => {
                        println!("Account <{}> doesn't exist", cli_owner_account_id);
                        Sender::input_owner_account_id(&context)
                    }
                },
                None => Ok(cli_owner_account_id),
            },
            None => Self::input_owner_account_id(&context),
        }
    }

    fn input_owner_account_id(
        context: &super::operation_mode::AddSubAccountCommandNetworkContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        loop {
            let account_id: crate::types::account_id::AccountId = Input::new()
                .with_prompt("What is the owner account ID?")
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
        let unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self.owner_account_id.clone().into(),
            receiver_id: self.owner_account_id.clone().into(),
            ..prepopulated_unsigned_transaction
        };
        self.sub_account
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
