use dialoguer::Input;
use near_primitives::borsh::BorshDeserialize;
use near_primitives::borsh::BorshSerialize;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug)]
pub struct SignTransaction {
    pub signer_secret_key: String,
    // transaction_format: crate::common::TransactionFormat,
    pub unsigned_transaction: String,
}

#[derive(Debug, StructOpt)]
pub struct CliSignTransaction {
    #[structopt(long)]
    signer_secret_key: Option<String>,
    #[structopt(long)]
    unsigned_transaction: Option<String>,
}

impl From<CliSignTransaction> for SignTransaction {
    fn from(item: CliSignTransaction) -> Self {
        let signer_secret_key: String = match item.signer_secret_key {
            Some(cli_signer_secret_key) => cli_signer_secret_key,
            None => SignTransaction::input_signer_secret_key(),
        };
        let unsigned_transaction: String = match item.unsigned_transaction {
            Some(cli_unsigned_transaction) => cli_unsigned_transaction,
            None => SignTransaction::input_unsigned_transaction(),
        };
        SignTransaction {
            signer_secret_key,
            unsigned_transaction,
        }
    }
}

impl SignTransaction {
    pub fn process(self) {
        let unsigned_transaction_borsh = base64::decode(&self.unsigned_transaction).unwrap();
        let unsigned_transaction =
            near_primitives::transaction::Transaction::try_from_slice(&unsigned_transaction_borsh)
                .unwrap();
        let signer_secret_key = near_crypto::SecretKey::from_str(&self.signer_secret_key).unwrap();
        let signature = signer_secret_key.sign(&unsigned_transaction.get_hash().as_ref());
        let signed_transaction =
            near_primitives::transaction::SignedTransaction::new(signature, unsigned_transaction);
        let serialize_to_base64 = near_primitives::serialize::to_base64(
            signed_transaction
                .try_to_vec()
                .expect("Transaction is not expected to fail on serialization"),
        );
        println!(
            "The transaction has been successfully signed:\n{:#?}",
            signed_transaction
        );
        println!("Base64-encoded signed transaction: {}", serialize_to_base64);
    }
    pub fn input_signer_secret_key() -> String {
        Input::new()
            .with_prompt("Enter the secret key")
            .interact_text()
            .unwrap()
    }
    pub fn input_unsigned_transaction() -> String {
        Input::new()
            .with_prompt("Enter an unsigned transaction")
            .interact_text()
            .unwrap()
    }
}
