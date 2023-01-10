use inquire::{CustomType, Select};

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
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    faucet_url: Option<crate::types::url::Url>,
}

impl interactive_clap::FromCli for AddNetworkConnection {
    type FromCliContext = crate::GlobalContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<
            <AddNetworkConnection as interactive_clap::ToCli>::CliVariant,
        >,
        context: Self::FromCliContext,
    ) -> Result<Option<Self>, Self::FromCliError>
    where
        Self: Sized + interactive_clap::ToCli,
    {
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
            match optional_clap_variant
                .as_ref()
                .and_then(|clap_variant| clap_variant.linkdrop_account_id.clone())
            {
                Some(cli_linkdrop_account_id) => Some(cli_linkdrop_account_id),
                None => {
                    println!();
                    #[derive(strum_macros::Display)]
                    enum ConfirmOptions {
                        #[strum(
                            to_string = "Yes, and I want to enter the name of the account hosting the program \"linkdrop\""
                        )]
                        Yes,
                        #[strum(to_string = "I dont know")]
                        No,
                    }
                    let select_choose_input = Select::new(
                        "Is there a \"linkdrop\" program on this network?",
                        vec![ConfirmOptions::Yes, ConfirmOptions::No],
                    )
                    .prompt()?;
                    if let ConfirmOptions::Yes = select_choose_input {
                        Self::input_linkdrop_account_id()?
                    } else {
                        None
                    }
                }
            };
        let faucet_url: Option<crate::types::url::Url> =
            match optional_clap_variant.and_then(|clap_variant| clap_variant.faucet_url) {
                Some(cli_faucet_url) => Some(cli_faucet_url),
                None => {
                    println!();
                    #[derive(strum_macros::Display)]
                    enum ConfirmOptions {
                        #[strum(to_string = "Yes, I want to enter the URL of the faucet")]
                        Yes,
                        #[strum(to_string = "No, I don't want to enter the faucet URL")]
                        No,
                    }
                    let select_choose_input = Select::new(
                        "Do you want to enter the faucet URL?",
                        vec![ConfirmOptions::Yes, ConfirmOptions::No],
                    )
                    .prompt()?;
                    if let ConfirmOptions::Yes = select_choose_input {
                        Self::input_faucet_url()?
                    } else {
                        None
                    }
                }
            };
        Ok(Some(Self {
            network_name,
            connection_name,
            rpc_url,
            wallet_url,
            explorer_transaction_url,
            rpc_api_key,
            linkdrop_account_id,
            faucet_url,
        }))
    }
}

impl AddNetworkConnection {
    fn input_api_key() -> color_eyre::eyre::Result<Option<crate::types::api_key::ApiKey>> {
        println!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, the RPC endpoint requires API key")]
            Yes,
            #[strum(to_string = "No, the RPC endpoint does not require API key")]
            No,
        }
        let select_choose_input = Select::new(
            "Do you want to input an API key?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let api_key: crate::types::api_key::ApiKey =
                CustomType::new("Enter an API key").prompt()?;
            Ok(Some(api_key))
        } else {
            Ok(None)
        }
    }

    fn input_linkdrop_account_id(
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        let account_id: crate::types::account_id::AccountId =
            CustomType::new("What is the name of the account that hosts the \"linkdrop\" program? (e.g. on mainnet it is near, and on testnet it is testnet)").prompt()?;
        Ok(Some(account_id))
    }

    fn input_faucet_url() -> color_eyre::eyre::Result<Option<crate::types::url::Url>> {
        let faucet_url: crate::types::url::Url =
            CustomType::new("What is the faucet url?").prompt()?;
        Ok(Some(faucet_url))
    }

    pub async fn process(&self, mut config: crate::config::Config) -> crate::CliResult {
        config.networks.insert(
            self.connection_name.clone(),
            crate::config::NetworkConfig {
                network_name: self.network_name.clone(),
                rpc_url: self.rpc_url.clone().into(),
                wallet_url: self.wallet_url.clone().into(),
                explorer_transaction_url: self.explorer_transaction_url.0.clone(),
                rpc_api_key: self.rpc_api_key.clone(),
                linkdrop_account_id: self
                    .linkdrop_account_id
                    .clone()
                    .map(|linkdrop_account_id| linkdrop_account_id.into()),
                faucet_url: self.faucet_url.clone().map(|faucet_url| faucet_url.into()),
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
