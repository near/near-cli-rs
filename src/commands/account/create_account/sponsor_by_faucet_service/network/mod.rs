#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct Network {
    ///What is the name of the network
    #[interactive_clap(skip_default_input_arg)]
    network_name: String,
    #[interactive_clap(subcommand)]
    pub submit: Option<crate::transaction_signature_options::Submit>,
}

impl interactive_clap::FromCli for Network {
    type FromCliContext = crate::GlobalContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<Network as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> Result<Option<Self>, Self::FromCliError>
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let network_name: String = match optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.network_name.as_ref())
        {
            Some(network_name) => network_name.clone(),
            None => Self::input_network_name(&context)?,
        };
        let submit: Option<crate::transaction_signature_options::Submit> =
            optional_clap_variant.and_then(|clap_variant| clap_variant.submit);
        Ok(Some(Self {
            network_name,
            submit,
        }))
    }
}

impl Network {
    fn input_network_name(context: &crate::GlobalContext) -> color_eyre::eyre::Result<String> {
        crate::common::input_network_name(context)
    }

    pub fn get_network_config(
        &self,
        config: crate::config::Config,
    ) -> crate::config::NetworkConfig {
        let network_config = config.networks;
        network_config
            .get(self.network_name.as_str())
            .expect("Impossible to get network name!")
            .clone()
    }

    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: crate::commands::account::create_account::AccountProperties,
        storage_message: Option<String>,
    ) -> crate::CliResult {
        let network_config = self.get_network_config(config.clone());

        println!("\nYour transaction:");
        println!("{:<13} {}", "signer_id:", &network_config.network_name);
        println!("actions:");
        println!(
            "{:>5} {:<20} {}",
            "--", "create account:", &account_properties.new_account_id
        );
        println!("{:>5} {:<20}", "--", "add access key:");
        println!(
            "{:>18} {:<13} {}",
            "", "public key:", &account_properties.public_key
        );
        println!("{:>18} {:<13} FullAccess", "", "permission:");
        println!();

        let submit = match self.submit.clone() {
            None => crate::transaction_signature_options::Submit::choose_submit(),
            Some(submit) => submit,
        };
        match submit {
            crate::transaction_signature_options::Submit::Send => {
                if let Some(message) = storage_message {
                    println!("{}\n", message);
                }
                let faucet_service_url = match &network_config.faucet_url {
                    Some(url) => url,
                    None => return Err(color_eyre::Report::msg(format!(
                        "The <{}> network does not have a faucet (helper service) that can sponsor the creation of an account.",
                        network_config.network_name
                    )))
                };
                let mut data = std::collections::HashMap::new();
                data.insert(
                    "newAccountId",
                    account_properties.new_account_id.to_string(),
                );
                data.insert(
                    "newAccountPublicKey",
                    account_properties.public_key.to_string(),
                );

                let client = reqwest::Client::new();
                match client
                    .post(faucet_service_url.clone())
                    .json(&data)
                    .send()
                    .await
                {
                    Ok(response) => {
                        let account_creation_transaction = response
                            .json::<near_jsonrpc_client::methods::tx::RpcTransactionStatusResponse>(
                            )
                            .await?;
                        match account_creation_transaction.status {
                            near_primitives::views::FinalExecutionStatus::SuccessValue(
                                ref value,
                            ) => {
                                if value == b"false" {
                                    println!(
                                        "The new account <{}> could not be created successfully.",
                                        account_properties.new_account_id
                                    );
                                } else {
                                    println!(
                                        "New account <{}> created successfully.",
                                        account_properties.new_account_id
                                    );
                                }
                                println!("Transaction ID: {id}\nTo see the transaction in the transaction explorer, please open this url in your browser:\n{path}{id}\n",
                                    id=account_creation_transaction.transaction_outcome.id,
                                    path=network_config.explorer_transaction_url
                                );
                                Ok(())
                            }
                            _ => {
                                crate::common::print_transaction_status(
                                    account_creation_transaction,
                                    network_config,
                                )?;
                                Ok(())
                            }
                        }
                    }
                    Err(err) => Err(color_eyre::Report::msg(err.to_string())),
                }
            }
            crate::transaction_signature_options::Submit::Display => Ok(()),
        }
    }
}
