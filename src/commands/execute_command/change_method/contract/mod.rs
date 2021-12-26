use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::operation_mode::ExecuteCommandNetworkContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct Contract {
    pub contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// вызов метода изменения
    pub call: super::call_function_type::CallFunctionAction,
}

impl Contract {
    pub fn from_cli(
        optional_clap_variant: Option<<Contract as interactive_clap::ToCli>::CliVariant>,
        context: super::operation_mode::ExecuteChangeMethodCommandNetworkContext,
    ) -> color_eyre::eyre::Result<Self> {
        let connection_config = context.connection_config.clone();
        let contract_account_id = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.contract_account_id)
        {
            Some(contract_account_id) => match &connection_config {
                Some(network_connection_config) => match crate::common::get_account_state(
                    &network_connection_config,
                    contract_account_id.clone().into(),
                )? {
                    Some(_) => contract_account_id,
                    None => {
                        println!("Account <{}> doesn't exist", contract_account_id);
                        Self::input_contract_account_id(&context)?
                    }
                },
                None => contract_account_id,
            },
            None => Self::input_contract_account_id(&context)?,
        };
        let call = super::call_function_type::CallFunctionAction::from_cli(
            optional_clap_variant.and_then(|clap_variant| match clap_variant.call {
                Some(ClapNamedArgCallFunctionActionForContract::Call(cli_args)) => Some(cli_args),
                None => None,
            }),
            context,
        )?;
        Ok(Self {
            contract_account_id,
            call,
        })
    }
}

impl Contract {
    fn input_contract_account_id(
        context: &super::operation_mode::ExecuteChangeMethodCommandNetworkContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        let connection_config = context.connection_config.clone();
        loop {
            let account_id: crate::types::account_id::AccountId = Input::new()
                .with_prompt("What is the account ID of the contract?")
                .interact_text()?;
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
            receiver_id: self.contract_account_id.clone().into(),
            ..prepopulated_unsigned_transaction
        };
        self.call
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
