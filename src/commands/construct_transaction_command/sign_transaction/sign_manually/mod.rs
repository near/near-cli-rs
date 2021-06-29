use dialoguer::Input;
use near_primitives::borsh::BorshSerialize;

/// подписание сформированной транзакции в режиме manually
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliSignManually {
    #[clap(long)]
    signer_public_key: Option<near_crypto::PublicKey>,
}

#[derive(Debug)]
pub struct SignManually {
    pub signer_public_key: near_crypto::PublicKey,
}

impl From<CliSignManually> for SignManually {
    fn from(item: CliSignManually) -> Self {
        let signer_public_key: near_crypto::PublicKey = match item.signer_public_key {
            Some(cli_public_key) => cli_public_key,
            None => SignManually::signer_public_key(),
        };
        SignManually { signer_public_key }
    }
}

impl SignManually {
    pub fn signer_public_key() -> near_crypto::PublicKey {
        Input::new()
            .with_prompt("To create an unsigned transaction enter sender's public key")
            .interact_text()
            .unwrap()
    }

    fn rpc_client(self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        let public_key: near_crypto::PublicKey = self.signer_public_key.clone();

        let unsigned_transaction = match network_connection_config {
            None => near_primitives::transaction::Transaction {
                public_key,
                ..prepopulated_unsigned_transaction
            },
            Some(network_connection_config) => {
                let online_signer_access_key_response = self
                    .rpc_client(network_connection_config.rpc_url().as_str())
                    .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
                        block_reference: near_primitives::types::Finality::Final.into(),
                        request: near_primitives::views::QueryRequest::ViewAccessKey {
                            account_id: prepopulated_unsigned_transaction.signer_id.clone(),
                            public_key: public_key.clone(),
                        },
                    })
                    .await
                    .map_err(|err| {
                        color_eyre::Report::msg(format!(
                            "Failed to fetch public key information for nonce: {:?}",
                            err
                        ))
                    })?;
                let current_nonce =
                    if let near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKey(
                        online_signer_access_key,
                    ) = online_signer_access_key_response.kind
                    {
                        online_signer_access_key.nonce
                    } else {
                        return Err(color_eyre::Report::msg(format!("Error current_nonce")));
                    };
                near_primitives::transaction::Transaction {
                    public_key,
                    block_hash: online_signer_access_key_response.block_hash,
                    nonce: current_nonce + 1,
                    ..prepopulated_unsigned_transaction
                }
            }
        };

        println!();
        println!("Unsigned transaction:\n\n {:#?}", &unsigned_transaction);
        let serialize_to_base64 = near_primitives::serialize::to_base64(
            unsigned_transaction
                .try_to_vec()
                .expect("Transaction is not expected to fail on serialization"),
        );
        println!(
            "---  serialize_to_base64:   --- \n   {:#?}",
            &serialize_to_base64
        );
        Ok(None)
    }
}
