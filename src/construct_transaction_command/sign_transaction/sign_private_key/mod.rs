use near_primitives::borsh::BorshSerialize;
use std::str::FromStr;
use structopt::StructOpt;
use dialoguer::{
    Input,
};


#[derive(Debug)]
pub struct SignPrivateKey {
    pub signer_public_key: String,
    pub signer_secret_key: String,
}

#[derive(Debug, StructOpt)]
pub struct CliSignPrivateKey {
    #[structopt(long)]
    signer_public_key: Option<String>,
    #[structopt(long)]
    signer_secret_key: Option<String>
}

impl SignPrivateKey {
    fn rpc_client(self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: String,
    ) {
        println!("SignPrivateKey process: self:\n       {:?}", &self);
        println!("SignPrivateKey process: prepopulated_unsigned_transaction:\n       {:?}", &prepopulated_unsigned_transaction);
        println!("SignPrivateKey process: selected_server_url:\n       {:?}", &selected_server_url);
        let public_key = near_crypto::PublicKey::from_str(&self.signer_public_key).unwrap();
        let signer_secret_key = near_crypto::SecretKey::from_str(&self.signer_secret_key).unwrap();
        if selected_server_url.is_empty() {
            let unsigned_transaction = near_primitives::transaction::Transaction {
                public_key,
                .. prepopulated_unsigned_transaction
            };
            let signature = signer_secret_key
                .sign(unsigned_transaction.get_hash().as_ref());
            let signed_transaction =
                near_primitives::transaction::SignedTransaction::new(signature, unsigned_transaction);
            let serialize_to_base64 = near_primitives::serialize::to_base64(
                    signed_transaction
                        .try_to_vec()
                        .expect("Transaction is not expected to fail on serialization"),
                );
            println!("---  serialize_to_base64:   --- \n   {:#?}", &serialize_to_base64)
            }
        else {
            let online_signer_access_key_response = self
                .rpc_client(&selected_server_url)
                .query(near_primitives::rpc::RpcQueryRequest {
                    block_reference: near_primitives::types::Finality::Final.into(),
                    request: near_primitives::views::QueryRequest::ViewAccessKey {
                        account_id: prepopulated_unsigned_transaction.signer_id.clone(),
                        public_key: public_key.clone(),
                    },
                })
                .await
                .map_err(|err| {
                    println!("Error online_signer_access_key_response:   {:?}", &err)
                }).unwrap();
            println!("online_signer_access_key_response:\n   {:?}", &online_signer_access_key_response);
            let current_nonce =
                if let near_primitives::views::QueryResponseKind::AccessKey(online_signer_access_key) =
                    online_signer_access_key_response.kind
                {
                    online_signer_access_key.nonce
                } else {
                    return println!("Error current_nonce");
                };
            println!("current_nonce:  {:?}", &current_nonce);
            let unsigned_transaction = near_primitives::transaction::Transaction {
                public_key,
                block_hash: online_signer_access_key_response.block_hash,
                nonce: current_nonce + 1,
                .. prepopulated_unsigned_transaction
            };
            println!("unsigned_transaction:  {:#?}", &unsigned_transaction);
            let signature = signer_secret_key
                .sign(unsigned_transaction.get_hash().as_ref());
            let signed_transaction =
                near_primitives::transaction::SignedTransaction::new(signature, unsigned_transaction);
            println!("---  Signed transaction:   ---    {:#?}", &signed_transaction);
            let transaction_info = near_jsonrpc_client::new_client(&selected_server_url)
                .broadcast_tx_commit(near_primitives::serialize::to_base64(
                    signed_transaction
                        .try_to_vec()
                        .expect("Transaction is not expected to fail on serialization"),
                ))
                .await
                .map_err(|err| {
                    println!("Error transaction:  {:?}",&err)
                })
                .unwrap();
            println!("Success: {:#?}", transaction_info);
            }
    }
    pub fn signer_public_key() -> String {
        Input::new()
            .with_prompt("enter sender's public key")
            .interact_text()
            .unwrap()
    }
    pub fn signer_secret_key() -> String {
        Input::new()
            .with_prompt("enter sender's private key")
            .interact_text()
            .unwrap()
    }
}

impl From<CliSignPrivateKey> for SignPrivateKey {
    fn from(item: CliSignPrivateKey) -> Self {
        let signer_public_key: String = match item.signer_public_key {
            Some(cli_public_key) => cli_public_key,
            None => SignPrivateKey::signer_public_key()
        };
        let signer_secret_key: String = match item.signer_secret_key {
            Some(cli_secret_key) => cli_secret_key,
            None => SignPrivateKey::signer_secret_key()
        };
        SignPrivateKey {
            signer_public_key,
            signer_secret_key
        }
    }
}
