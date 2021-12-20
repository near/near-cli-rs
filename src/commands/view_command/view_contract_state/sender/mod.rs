use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::operation_mode::online_mode::select_server::ViewContractStateCommandNetworkContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct Sender {
    pub sender_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    selected_block_id: super::block_id::BlockId,
}

impl Sender {
    pub fn from_cli(
        optional_clap_variant: Option<CliSender>,
        context: super::operation_mode::online_mode::select_server::ViewContractStateCommandNetworkContext,
    ) -> color_eyre::eyre::Result<Self> {
        let connection_config = context.connection_config.clone();
        let sender_account_id = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.sender_account_id)
        {
            Some(sender_account_id) => match crate::common::get_account_state(
                &connection_config,
                sender_account_id.clone().into(),
            )? {
                Some(_) => sender_account_id,
                None => {
                    println!("Contract <{}> doesn't exist", sender_account_id);
                    Self::input_sender_account_id(&context)?
                }
            },
            None => Self::input_sender_account_id(&context)?,
        };
        let selected_block_id: super::block_id::BlockId = match optional_clap_variant
            .and_then(|clap_variant| clap_variant.selected_block_id)
        {
            Some(cli_block_id) => super::block_id::BlockId::from_cli(Some(cli_block_id), context)?,
            None => super::block_id::BlockId::choose_variant(context)?,
        };
        Ok(Self {
            sender_account_id,
            selected_block_id,
        })
    }
}

impl Sender {
    pub fn input_sender_account_id(
        context: &super::operation_mode::online_mode::select_server::ViewContractStateCommandNetworkContext,
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
        self.selected_block_id
            .process(self.sender_account_id.into(), network_connection_config)
            .await
    }
}
