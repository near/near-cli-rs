use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::operation_mode::online_mode::select_server::ExecuteViewMethodCommandNetworkContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct Contract {
    pub contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// calling a view method
    pub call: super::call_function_type::CallFunctionView,
}

impl Contract {
    pub fn from_cli(
        optional_clap_variant: Option<<Contract as interactive_clap::ToCli>::CliVariant>,
        context: super::operation_mode::online_mode::select_server::ExecuteViewMethodCommandNetworkContext,
    ) -> color_eyre::eyre::Result<Self> {
        let connection_config = context.connection_config.clone();
        let contract_account_id = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.contract_account_id)
        {
            Some(contract_account_id) => match crate::common::get_account_state(
                &connection_config,
                contract_account_id.clone().into(),
            )? {
                Some(_) => contract_account_id,
                None => {
                    println!("Account <{}> doesn't exist", contract_account_id);
                    Self::input_contract_account_id(&context)?
                }
            },
            None => Self::input_contract_account_id(&context)?,
        };
        let call = super::call_function_type::CallFunctionView::from_cli(
            optional_clap_variant.and_then(|clap_variant| match clap_variant.call {
                Some(ClapNamedArgCallFunctionViewForContract::Call(cli_args)) => Some(cli_args),
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
        context: &super::operation_mode::online_mode::select_server::ExecuteViewMethodCommandNetworkContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        let connection_config = context.connection_config.clone();
        loop {
            let contract_account_id: crate::types::account_id::AccountId = Input::new()
                .with_prompt("What is the account ID of the contract?")
                .interact_text()?;
            let contract_code_hash: near_primitives::hash::CryptoHash =
                match crate::common::get_account_state(
                    &connection_config,
                    contract_account_id.clone().into(),
                )? {
                    Some(account_view) => account_view.code_hash,
                    None => near_primitives::hash::CryptoHash::default(),
                };
            if contract_code_hash == near_primitives::hash::CryptoHash::default() {
                println!(
                    "Contract code is not deployed to this account <{}>.",
                    contract_account_id.to_string()
                )
            } else {
                break Ok(contract_account_id);
            }
        }
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        self.call
            .process(network_connection_config, self.contract_account_id.into())
            .await
    }
}
