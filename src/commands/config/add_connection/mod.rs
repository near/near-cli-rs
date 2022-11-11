use dialoguer::{console::Term, theme::ColorfulTheme, Input, Select};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct AddNetworkConnection {
    #[interactive_clap(long)]
    ///What is the NEAR network? (e.g. mainnet, testnet, shardnet)
    network_name: String,
    #[interactive_clap(long)]
    ///What is the connection name? (e.g. pagoda-mainnet)
    connection_name: String,
    #[interactive_clap(long)]
    ///What is the RPC endpoint?
    rpc_url: crate::types::url::Url,
    #[interactive_clap(long)]
    ///What is the wallet endpoint?
    wallet_url: crate::types::url::Url,
    #[interactive_clap(long)]
    ///What is the transaction explorer endpoint?
    explorer_transaction_url: crate::types::url::Url,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    rpc_api_key: Option<crate::types::api_key::ApiKey>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    linkdrop_account_id: Option<crate::types::account_id::AccountId>,
}

impl AddNetworkConnection {
    pub fn from_cli(
        optional_clap_variant: Option<
            <AddNetworkConnection as interactive_clap::ToCli>::CliVariant,
        >,
        context: crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<Self>> {
        let network_name = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.network_name)
        {
            Some(cli_network_name) => cli_network_name,
            None => Self::input_network_name(&context)?,
        };
        let connection_name = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.connection_name)
        {
            Some(cli_connection_name) => cli_connection_name,
            None => Self::input_connection_name(&context)?,
        };
        let rpc_url = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.rpc_url)
        {
            Some(cli_rpc_url) => cli_rpc_url,
            None => Self::input_rpc_url(&context)?,
        };
        let wallet_url = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.wallet_url)
        {
            Some(cli_wallet_url) => cli_wallet_url,
            None => Self::input_wallet_url(&context)?,
        };
        let explorer_transaction_url = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.explorer_transaction_url)
        {
            Some(cli_explorer_transaction_url) => cli_explorer_transaction_url,
            None => Self::input_explorer_transaction_url(&context)?,
        };
        let rpc_api_key: Option<crate::types::api_key::ApiKey> = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.rpc_api_key)
        {
            Some(cli_api_key) => Some(cli_api_key),
            None => Self::input_api_key()?,
        };
        let linkdrop_account_id: Option<crate::types::account_id::AccountId> =
            match optional_clap_variant.and_then(|clap_variant| clap_variant.linkdrop_account_id) {
                Some(cli_linkdrop_account_id) => Some(cli_linkdrop_account_id),
                None => Self::input_linkdrop_account_id()?,
            };
        Ok(Some(Self {
            network_name,
            connection_name,
            rpc_url,
            wallet_url,
            explorer_transaction_url,
            rpc_api_key,
            linkdrop_account_id,
        }))
    }

    fn input_api_key() -> color_eyre::eyre::Result<Option<crate::types::api_key::ApiKey>> {
        println!();
        let choose_input = vec![
            "Yes, the RPC endpoint requires API key",
            "No, the RPC endpoint does not require API key",
        ];
        let select_choose_input = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to input an API key?")
            .items(&choose_input)
            .default(0)
            .interact_on_opt(&Term::stderr())?;
        match select_choose_input {
            Some(0) => {
                let api_key = Input::new()
                    .with_prompt("Enter an API key")
                    .interact_text()?;
                Ok(Some(api_key))
            }
            Some(1) => Ok(None),
            _ => unreachable!("Error"),
        }
    }

    fn input_linkdrop_account_id(
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        let account_id: crate::types::account_id::AccountId = Input::new()
            .with_prompt(
                "What is the name of the account that hosts the \"linkdrop\" program?(e.g. on mainnet it is near, and on testnet it is testnet)",
            )
            .interact_text()?;
        Ok(Some(account_id))
    }

    pub async fn process(&self, mut config: crate::config::Config) -> crate::CliResult {
        config.networks.insert(
            self.connection_name.clone(),
            crate::config::NetworkConfig {
                network_name: self.network_name.clone(),
                rpc_url: self.rpc_url.0.clone(),
                wallet_url: self.wallet_url.0.clone(),
                explorer_transaction_url: self.explorer_transaction_url.0.clone(),
                rpc_api_key: self.rpc_api_key.clone(),
                linkdrop_account_id: self
                    .linkdrop_account_id
                    .clone()
                    .map(|linkdrop_account_id| linkdrop_account_id.into()),
            },
        );
        println!();
        crate::common::write_config_toml(config)?;
        println!(
            "Network connection \"{}\" was successfully added to config.toml",
            &self.connection_name
        );
        Ok(())
    }
}
