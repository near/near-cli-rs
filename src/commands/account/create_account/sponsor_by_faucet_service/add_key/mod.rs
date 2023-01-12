use strum::{EnumDiscriminants, EnumIter, EnumMessage};

// pub mod autogenerate_new_keypair;
// #[cfg(feature = "ledger")]
// mod use_ledger;
// mod use_manually_provided_seed_phrase;
mod use_public_key;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///Add an access key for this account
pub enum AccessKeyMode {
    // #[strum_discriminants(strum(
    //     message = "autogenerate-new-keypair          - Automatically generate a key pair"
    // ))]
    // ///Automatically generate a key pair
    // AutogenerateNewKeypair(self::autogenerate_new_keypair::GenerateKeypair),
    // #[strum_discriminants(strum(
    //     message = "use-manually-provided-seed-prase  - Use the provided seed phrase manually"
    // ))]
    // ///Use the provided seed phrase manually
    // UseManuallyProvidedSeedPhrase(
    //     self::use_manually_provided_seed_phrase::AddAccessWithSeedPhraseAction,
    // ),
    #[strum_discriminants(strum(
        message = "use-manually-provided-public-key  - Use the provided public key manually"
    ))]
    ///Use the provided public key manually
    UseManuallyProvidedPublicKey(self::use_public_key::AddAccessKeyAction),
    //     #[cfg(feature = "ledger")]
    //     #[strum_discriminants(strum(message = "use-ledger                        - Use a ledger"))]
    //     ///Use a ledger
    //     UseLedger(self::use_ledger::AddAccessWithLedger),
}

impl AccessKeyMode {
    pub async fn process(
        &self,
        config: crate::config::Config,
        new_account_id: crate::types::account_id::AccountId,
    ) -> crate::CliResult {
        let (public_key, network_config) = match self {
            AccessKeyMode::UseManuallyProvidedPublicKey(add_access_key_action) => (
                add_access_key_action.get_public_key(),
                add_access_key_action.get_network_config(config),
            ), // AccessKeyMode::AutogenerateNewKeypair(generate_keypair) => {
               //     generate_keypair.process(config, account_properties).await
               // }
               // AccessKeyMode::UseManuallyProvidedSeedPhrase(add_access_with_seed_phrase_action) => {
               //     add_access_with_seed_phrase_action
               //         .process(config, account_properties)
               //         .await
               // }
               // #[cfg(feature = "ledger")]
               // AccessKeyMode::UseLedger(add_access_with_ledger) => {
               //     add_access_with_ledger
               //         .process(config, account_properties)
               //         .await
               // }
        };
        let faucet_service_url = match network_config.faucet_url {
            Some(url) => url,
            None => return Err(color_eyre::Report::msg(format!(
                "Error: The <{}> network does not have a faucet (helper service) that can sponsor the creation of an account.",
                network_config.network_name
            )))
        };
        let mut data = std::collections::HashMap::new();
        data.insert("newAccountId", new_account_id.to_string());
        data.insert("newAccountPublicKey", public_key.to_string());

        let client = reqwest::Client::new();
        match client.post(faucet_service_url).json(&data).send().await {
            Ok(response) => {
                let account_creation_transaction = response
                    .json::<near_jsonrpc_client::methods::tx::RpcTransactionStatusResponse>()
                    .await?;
                match account_creation_transaction.status {
                    near_primitives::views::FinalExecutionStatus::SuccessValue(ref value) => {
                        if value == b"false" {
                            println!(
                                "The new account <{}> could not be created successfully.",
                                new_account_id
                            );
                        } else {
                            println!("New account <{}> created successfully.", new_account_id);
                        }
                        println!("Transaction ID: {id}\nTo see the transaction in the transaction explorer, please open this url in your browser:\n{path}{id}\n",
                                    id=account_creation_transaction.transaction_outcome.id,
                                    path=network_config.explorer_transaction_url
                                );
                        // if storage_properties.is_some() {
                        //     println!("{}\n", storage_message);
                        // }
                        Ok(())
                    }
                    near_primitives::views::FinalExecutionStatus::Failure(tx_execution_error) => {
                        match tx_execution_error {
                            near_primitives::errors::TxExecutionError::ActionError(
                                _action_error,
                            ) => todo!(),
                            near_primitives::errors::TxExecutionError::InvalidTxError(_) => todo!(),
                        }
                    }
                    near_primitives::views::FinalExecutionStatus::NotStarted => todo!(),
                    near_primitives::views::FinalExecutionStatus::Started => todo!(),
                }
            }
            Err(err) => Err(color_eyre::Report::msg(err.to_string())),
        }
    }
}
