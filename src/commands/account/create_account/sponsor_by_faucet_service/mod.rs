mod add_key;
mod network;

#[derive(Clone)]
pub struct SponsorServiceContext {
    pub config: crate::config::Config,
    pub new_account_id: crate::types::account_id::AccountId,
    pub public_key: near_crypto::PublicKey,
    pub on_after_getting_network_callback: self::network::OnAfterGettingNetworkCallback,
    pub on_before_creating_account_callback: self::network::OnBeforeCreatingAccountCallback,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = NewAccountContext)]
pub struct NewAccount {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the new account ID?
    new_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    access_key_mode: add_key::AccessKeyMode,
}

#[derive(Clone)]
pub struct NewAccountContext {
    config: crate::config::Config,
    new_account_id: crate::types::account_id::AccountId,
    on_before_creating_account_callback: self::network::OnBeforeCreatingAccountCallback,
}

impl NewAccountContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<NewAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let credentials_home_dir = previous_context.config.credentials_home_dir.clone();
        let on_before_creating_account_callback: self::network::OnBeforeCreatingAccountCallback =
            std::sync::Arc::new({
                move |network_config, new_account_id, public_key| {
                    let faucet_service_url = match &network_config.faucet_url {
                        Some(url) => url,
                        None => return Err(color_eyre::Report::msg(format!(
                            "The <{}> network does not have a faucet (helper service) that can sponsor the creation of an account.",
                            &network_config.network_name
                        )))
                    };
                    let mut data = std::collections::HashMap::new();
                    data.insert("newAccountId", new_account_id.to_string());
                    data.insert("newAccountPublicKey", public_key.to_string());

                    let client = reqwest::blocking::Client::new();
                    match client.post(faucet_service_url.clone()).json(&data).send() {
                        Ok(response) => {
                            let account_creation_transaction = response
                                .json::<near_jsonrpc_client::methods::tx::RpcTransactionStatusResponse>(
                                )?;
                            match account_creation_transaction.status {
                                near_primitives::views::FinalExecutionStatus::SuccessValue(
                                    ref value,
                                ) => {
                                    if value == b"false" {
                                        eprintln!(
                                        "The new account <{}> could not be created successfully.",
                                        &new_account_id
                                    );
                                    } else {
                                        crate::common::update_used_account_list_as_signer(
                                            &credentials_home_dir,
                                            new_account_id.as_ref(),
                                        );
                                        eprintln!(
                                            "New account <{}> created successfully.",
                                            &new_account_id
                                        );
                                    }
                                    eprintln!("Transaction ID: {id}\nTo see the transaction in the transaction explorer, please open this url in your browser:\n{path}{id}\n",
                                        id=account_creation_transaction.transaction_outcome.id,
                                        path=network_config.explorer_transaction_url
                                    );
                                }
                                _ => {
                                    crate::common::print_transaction_status(
                                        &account_creation_transaction,
                                        network_config,
                                    )?;
                                }
                            }
                            Ok(())
                        }
                        Err(err) => Err(color_eyre::Report::msg(err.to_string())),
                    }
                }
            });

        Ok(Self {
            config: previous_context.config,
            new_account_id: scope.new_account_id.clone(),
            on_before_creating_account_callback,
        })
    }
}

impl NewAccount {
    fn input_new_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        super::fund_myself_create_account::NewAccount::input_new_account_id(context)
    }
}
