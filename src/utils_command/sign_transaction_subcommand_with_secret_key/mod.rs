use dialoguer::Input;
use near_primitives::borsh::BorshSerialize;
use structopt::StructOpt;

#[derive(Debug)]
pub struct SignTransactionSecretKey {
    pub signer_secret_key: near_crypto::SecretKey,
    pub unsigned_transaction: near_primitives::transaction::Transaction,
}

#[derive(Debug, Default, StructOpt)]
pub struct CliSignTransactionSecretKey {
    #[structopt(long)]
    signer_secret_key: Option<near_crypto::SecretKey>,
    #[structopt(long)]
    unsigned_transaction: Option<crate::common::TransactionAsBase64>,
}

impl From<CliSignTransactionSecretKey> for SignTransactionSecretKey {
    fn from(item: CliSignTransactionSecretKey) -> Self {
        let signer_secret_key: near_crypto::SecretKey = match item.signer_secret_key {
            Some(cli_signer_secret_key) => cli_signer_secret_key,
            None => SignTransactionSecretKey::input_signer_secret_key(),
        };
        let unsigned_transaction: near_primitives::transaction::Transaction =
            match item.unsigned_transaction {
                Some(cli_unsigned_transaction) => cli_unsigned_transaction.inner,
                None => SignTransactionSecretKey::input_unsigned_transaction(),
            };
        SignTransactionSecretKey {
            signer_secret_key,
            unsigned_transaction,
        }
    }
}

impl SignTransactionSecretKey {
    pub fn process(self) {
        let signature = self
            .signer_secret_key
            .sign(&self.unsigned_transaction.get_hash().as_ref());
        println!("Signature:  {:?}", &signature);
        let signed_transaction = near_primitives::transaction::SignedTransaction::new(
            signature,
            self.unsigned_transaction,
        );
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
    pub fn input_signer_secret_key() -> near_crypto::SecretKey {
        Input::new()
            .with_prompt("Enter the secret key")
            .interact_text()
            .unwrap()
    }
    pub fn input_unsigned_transaction() -> near_primitives::transaction::Transaction {
        let input: crate::common::TransactionAsBase64 = Input::new()
            .with_prompt("Enter an unsigned transaction")
            .interact_text()
            .unwrap();
        input.inner
    }
}
