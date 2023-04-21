use inquire::{CustomType, Select};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = AddNetworkConnectionContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct AddNetworkConnection {
    #[interactive_clap(long)]
    /// What is the NEAR network? (e.g. mainnet, testnet, shardnet)
    network_name: String,
    #[interactive_clap(long)]
    /// What is the connection name? (e.g. pagoda-mainnet)
    connection_name: String,
    #[interactive_clap(long)]
    /// What is the RPC endpoint?
    rpc_url: crate::types::url::Url,
    #[interactive_clap(long)]
    /// What is the wallet endpoint?
    wallet_url: crate::types::url::Url,
    #[interactive_clap(long)]
    /// What is the transaction explorer endpoint?
    explorer_transaction_url: crate::types::url::Url,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    rpc_api_key: Option<crate::types::api_key::ApiKey>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    linkdrop_account_id: Option<crate::types::account_id::AccountId>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    faucet_url: Option<crate::types::url::Url>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    meta_transaction_relayer_url: Option<crate::types::url::Url>,
}

#[derive(Clone)]
pub struct AddNetworkConnectionContext;

impl AddNetworkConnectionContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<AddNetworkConnection as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let mut config = previous_context.0;
        config.network_connection.insert(
            scope.connection_name.clone(),
            crate::config::NetworkConfig {
                network_name: scope.network_name.clone(),
                rpc_url: scope.rpc_url.clone().into(),
                wallet_url: scope.wallet_url.clone().into(),
                explorer_transaction_url: scope.explorer_transaction_url.0.clone(),
                rpc_api_key: scope.rpc_api_key.clone(),
                linkdrop_account_id: scope
                    .linkdrop_account_id
                    .clone()
                    .map(|linkdrop_account_id| linkdrop_account_id.into()),
                faucet_url: scope.faucet_url.clone().map(|faucet_url| faucet_url.into()),
                meta_transaction_relayer_url: scope
                    .meta_transaction_relayer_url
                    .clone()
                    .map(|meta_transaction_relayer_url| meta_transaction_relayer_url.into()),
            },
        );
        eprintln!();
        crate::common::write_config_toml(config)?;
        eprintln!(
            "Network connection \"{}\" was successfully added to config.toml",
            &scope.connection_name
        );
        Ok(Self)
    }
}

impl interactive_clap::FromCli for AddNetworkConnection {
    type FromCliContext = crate::GlobalContext;
    type FromCliError = color_eyre::eyre::Error;
    fn from_cli(
        optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();
        if clap_variant.network_name.is_none() {
            clap_variant.network_name = match Self::input_network_name(&context) {
                Ok(Some(network_name)) => Some(network_name),
                Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let network_name = clap_variant.network_name.clone().expect("Unexpected error");
        if clap_variant.connection_name.is_none() {
            clap_variant.connection_name = match Self::input_connection_name(&context) {
                Ok(Some(connection_name)) => Some(connection_name),
                Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let connection_name = clap_variant
            .connection_name
            .clone()
            .expect("Unexpected error");
        if clap_variant.rpc_url.is_none() {
            clap_variant.rpc_url = match Self::input_rpc_url(&context) {
                Ok(Some(rpc_url)) => Some(rpc_url),
                Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let rpc_url = clap_variant.rpc_url.clone().expect("Unexpected error");
        if clap_variant.wallet_url.is_none() {
            clap_variant.wallet_url = match Self::input_wallet_url(&context) {
                Ok(Some(wallet_url)) => Some(wallet_url),
                Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let wallet_url = clap_variant.wallet_url.clone().expect("Unexpected error");
        if clap_variant.explorer_transaction_url.is_none() {
            clap_variant.explorer_transaction_url =
                match Self::input_explorer_transaction_url(&context) {
                    Ok(Some(explorer_transaction_url)) => Some(explorer_transaction_url),
                    Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                    Err(err) => {
                        return interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
                    }
                };
        };
        let explorer_transaction_url = clap_variant
            .explorer_transaction_url
            .clone()
            .expect("Unexpected error");
        if clap_variant.rpc_api_key.is_none() {
            clap_variant.rpc_api_key = match Self::input_rpc_api_key(&context) {
                Ok(optional_api_key) => optional_api_key,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let rpc_api_key = clap_variant.rpc_api_key.clone();
        if clap_variant.linkdrop_account_id.is_none() {
            clap_variant.linkdrop_account_id = match Self::input_linkdrop_account_id(&context) {
                Ok(optional_linkdrop_account_id) => optional_linkdrop_account_id,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let linkdrop_account_id = clap_variant.linkdrop_account_id.clone();
        if clap_variant.faucet_url.is_none() {
            clap_variant.faucet_url = match Self::input_faucet_url(&context) {
                Ok(optional_faucet_url) => optional_faucet_url,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let faucet_url = clap_variant.faucet_url.clone();
        if clap_variant.meta_transaction_relayer_url.is_none() {
            clap_variant.meta_transaction_relayer_url =
                match Self::input_meta_transaction_relayer_url(&context) {
                    Ok(optional_meta_transaction_relayer_url) => {
                        optional_meta_transaction_relayer_url
                    }
                    Err(err) => {
                        return interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
                    }
                };
        };
        let meta_transaction_relayer_url = clap_variant.meta_transaction_relayer_url.clone();
        let new_context_scope = InteractiveClapContextScopeForAddNetworkConnection {
            network_name,
            connection_name,
            rpc_url,
            wallet_url,
            explorer_transaction_url,
            rpc_api_key,
            linkdrop_account_id,
            faucet_url,
            meta_transaction_relayer_url,
        };
        if let Err(err) =
            AddNetworkConnectionContext::from_previous_context(context, &new_context_scope)
        {
            return interactive_clap::ResultFromCli::Err(Some(clap_variant), err);
        };
        interactive_clap::ResultFromCli::Ok(clap_variant)
    }
}

impl AddNetworkConnection {
    fn input_rpc_api_key(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::api_key::ApiKey>> {
        eprintln!();
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
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        eprintln!();
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
            let account_id: crate::types::account_id::AccountId =
            CustomType::new("What is the name of the account that hosts the \"linkdrop\" program? (e.g. on mainnet it is near, and on testnet it is testnet)").prompt()?;
            Ok(Some(account_id))
        } else {
            Ok(None)
        }
    }

    fn input_faucet_url(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::url::Url>> {
        eprintln!();
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
            let faucet_url: crate::types::url::Url =
                CustomType::new("What is the faucet url?").prompt()?;
            Ok(Some(faucet_url))
        } else {
            Ok(None)
        }
    }

    fn input_meta_transaction_relayer_url(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::url::Url>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, I want to enter the URL of the relayer")]
            Yes,
            #[strum(to_string = "No, I don't want to enter the relayer URL")]
            No,
        }
        let select_choose_input = Select::new(
            "Do you want to enter the meta transaction relayer URL?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let meta_transaction_relayer_url: crate::types::url::Url =
                CustomType::new("What is the relayer url?").prompt()?;
            Ok(Some(meta_transaction_relayer_url))
        } else {
            Ok(None)
        }
    }
}
