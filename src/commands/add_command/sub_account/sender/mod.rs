use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::operation_mode::AddSubAccountCommandNetworkContext)]
#[interactive_clap(output_context = crate::common::SenderContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct Sender {
    pub owner_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    ///Specify a sub-account
    pub sub_account: super::receiver::SubAccount,
}

impl crate::common::SenderContext {
    pub fn from_previous_context_for_add_sub_account(
        previous_context: super::operation_mode::AddSubAccountCommandNetworkContext,
        scope: &<Sender as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Self {
        Self {
            connection_config: previous_context.connection_config.clone(),
            sender_account_id: scope.owner_account_id.clone(),
        }
    }
}

impl Sender {
    pub fn from_cli(
        optional_clap_variant: Option<CliSender>,
        context: super::operation_mode::AddSubAccountCommandNetworkContext,
    ) -> color_eyre::eyre::Result<Self> {
        let connection_config = context.connection_config.clone();
        let owner_account_id = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.owner_account_id)
        {
            Some(sender_account_id) => match &connection_config {
                Some(network_connection_config) => match crate::common::get_account_state(
                    &network_connection_config,
                    sender_account_id.clone().into(),
                )? {
                    Some(_) => sender_account_id,
                    None => {
                        println!("Account <{}> doesn't exist", sender_account_id);
                        Sender::input_sub_account(&context)?
                    }
                },
                None => sender_account_id,
            },
            None => Self::input_sub_account(&context)?,
        };
        type Alias = <Sender as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope;
        let new_context_scope = Alias { owner_account_id };
        let new_context = crate::common::SenderContext::from_previous_context_for_add_sub_account(
            context,
            &new_context_scope,
        );
        let sub_account = super::receiver::SubAccount::from_cli(
            optional_clap_variant.and_then(|clap_variant| match clap_variant.sub_account {
                Some(ClapNamedArgSubAccountForSender::SubAccount(cli_sub_account)) => {
                    Some(cli_sub_account)
                }
                None => None,
            }),
            new_context,
        )?;
        Ok(Self {
            owner_account_id: new_context_scope.owner_account_id,
            sub_account,
        })
    }
}

impl Sender {
    fn input_sub_account(
        context: &super::operation_mode::AddSubAccountCommandNetworkContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        let connection_config = context.connection_config.clone();
        loop {
            let account_id: crate::types::account_id::AccountId = Input::new()
                .with_prompt("What is the owner account ID?")
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
