use dialoguer::Input;

mod download_mode;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::operation_mode::online_mode::select_server::ViewContractCodeCommandNetworkContext)]
#[interactive_clap(output_context = crate::common::SignerContext)]
pub struct Contract {
    #[interactive_clap(skip_default_from_cli)]
    pub contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    pub download_mode: self::download_mode::DownloadMode,
}

struct ContractContext {
    connection_config: Option<crate::common::ConnectionConfig>,
    contract_account_id: crate::types::account_id::AccountId,
}

impl ContractContext {
    pub fn from_previous_context(
        previous_context: super::operation_mode::online_mode::select_server::ViewContractCodeCommandNetworkContext,
        scope: &<Contract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Self {
        Self {
            connection_config: Some(previous_context.connection_config.clone()),
            contract_account_id: scope.contract_account_id.clone(),
        }
    }
}

impl From<ContractContext> for crate::common::SignerContext {
    fn from(item: ContractContext) -> Self {
        Self {
            connection_config: item.connection_config,
            signer_account_id: item.contract_account_id,
        }
    }
}

impl Contract {
    fn from_cli_contract_account_id(
        optional_cli_sender_account_id: Option<crate::types::account_id::AccountId>,
        context: &super::operation_mode::online_mode::select_server::ViewContractCodeCommandNetworkContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        match optional_cli_sender_account_id {
            Some(cli_sender_account_id) => match crate::common::get_account_state(
                &context.connection_config,
                cli_sender_account_id.clone().into(),
            )? {
                Some(_) => Ok(cli_sender_account_id),
                None => {
                    println!("Account <{}> doesn't exist", cli_sender_account_id);
                    Self::input_contract_account_id(&context)
                }
            },
            None => Self::input_contract_account_id(&context),
        }
    }

    pub fn input_contract_account_id(
        context: &super::operation_mode::online_mode::select_server::ViewContractCodeCommandNetworkContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        loop {
            let account_id: crate::types::account_id::AccountId = Input::new()
                .with_prompt("What contract do you need to view?")
                .interact_text()
                .unwrap();
            if let Some(_) = crate::common::get_account_state(
                &context.connection_config,
                account_id.clone().into(),
            )? {
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
