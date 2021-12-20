use dialoguer::Input;

mod download_mode;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::operation_mode::online_mode::select_server::ViewContractCodeCommandNetworkContext)]
#[interactive_clap(output_context = crate::common::SenderContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct Contract {
    pub contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    pub download_mode: self::download_mode::DownloadMode,
}

impl crate::common::SenderContext {
    pub fn from_previous_context_for_view_contract_code(
        previous_context: super::operation_mode::online_mode::select_server::ViewContractCodeCommandNetworkContext,
        scope: &<Contract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Self {
        Self {
            connection_config: Some(previous_context.connection_config.clone()),
            sender_account_id: scope.contract_account_id.clone(),
        }
    }
}

impl Contract {
    pub fn from_cli(
        optional_clap_variant: Option<CliContract>,
        context: super::operation_mode::online_mode::select_server::ViewContractCodeCommandNetworkContext,
    ) -> color_eyre::eyre::Result<Self> {
        let connection_config = context.connection_config.clone();
        let contract_account_id = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.contract_account_id)
        {
            Some(contract_id) => match crate::common::get_account_state(
                &connection_config,
                contract_id.clone().into(),
            )? {
                Some(_) => contract_id,
                None => {
                    println!("Contract <{}> doesn't exist", contract_id);
                    Self::input_contract_account_id(&context)?
                }
            },
            None => Self::input_contract_account_id(&context)?,
        };
        type Alias = <Contract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope;
        let new_context_scope = Alias {
            contract_account_id,
        };
        let new_context =
            crate::common::SenderContext::from_previous_context_for_view_contract_code(
                context,
                &new_context_scope,
            );
        let download_mode =
            match optional_clap_variant.and_then(|clap_variant| clap_variant.download_mode) {
                Some(cli_arg) => {
                    self::download_mode::DownloadMode::from_cli(Some(cli_arg), new_context)?
                }
                None => self::download_mode::DownloadMode::choose_variant(new_context)?,
            };
        Ok(Self {
            contract_account_id: new_context_scope.contract_account_id,
            download_mode,
        })
    }
}

impl Contract {
    pub fn input_contract_account_id(
        context: &super::operation_mode::online_mode::select_server::ViewContractCodeCommandNetworkContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        let connection_config = context.connection_config.clone();
        loop {
            let account_id: crate::types::account_id::AccountId = Input::new()
                .with_prompt("What contract do you need to view?")
                .interact_text()
                .unwrap();
            if let Some(_) =
                crate::common::get_account_state(&connection_config, account_id.clone().into())?
            {
                break Ok(account_id);
            } else {
                println!("Account <{}> doesn't exist", account_id.to_string());
            };
        }
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        self.download_mode
            .process(self.contract_account_id.into(), network_connection_config)
            .await
    }
}
