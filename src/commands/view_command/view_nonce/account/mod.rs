use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::operation_mode::online_mode::select_server::ViewNonceCommandNetworkContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct Account {
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    pub public_key: super::public_key::AccessKeyType,
}

impl Account {
    pub fn from_cli(
        optional_clap_variant: Option<CliAccount>,
        context: super::operation_mode::online_mode::select_server::ViewNonceCommandNetworkContext,
    ) -> color_eyre::eyre::Result<Self> {
        let connection_config = context.connection_config.clone();
        let account_id = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.account_id)
        {
            Some(account_id) => match crate::common::get_account_state(
                &connection_config,
                account_id.clone().into(),
            )? {
                Some(_) => account_id,
                None => {
                    println!("Contract <{}> doesn't exist", account_id);
                    Self::input_account_id(&context)?
                }
            },
            None => Self::input_account_id(&context)?,
        };
        let public_key = super::public_key::AccessKeyType::from_cli(
            optional_clap_variant.and_then(|clap_variant| match clap_variant.public_key {
                Some(ClapNamedArgAccessKeyTypeForAccount::PublicKey(cli_args)) => Some(cli_args),
                None => None,
            }),
            context,
        )?;
        Ok(Self {
            account_id,
            public_key,
        })
    }
}

impl Account {
    fn input_account_id(
        context: &super::operation_mode::online_mode::select_server::ViewNonceCommandNetworkContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        let connection_config = context.connection_config.clone();
        loop {
            let account_id: crate::types::account_id::AccountId = Input::new()
                .with_prompt("Enter your account ID")
                .interact_text()?;
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
        self.public_key
            .process(self.account_id.into(), network_connection_config)
            .await
    }
}
