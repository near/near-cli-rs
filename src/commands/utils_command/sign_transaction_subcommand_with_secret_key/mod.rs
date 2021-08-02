use dialoguer::Input;
use near_primitives::borsh::BorshSerialize;

/// утилита, позволяющая подписать транзакцию личным ключом
#[derive(Debug, Default, Clone, clap::Clap)]
pub struct CliSignTransactionSecretKey {
    #[clap(long)]
    signer_secret_key: Option<near_crypto::SecretKey>,
    #[clap(long)]
    unsigned_transaction: Option<crate::common::TransactionAsBase64>,
}

#[derive(Debug, Clone)]
pub struct SignTransactionSecretKey {
    pub signer_secret_key: near_crypto::SecretKey,
    pub unsigned_transaction: near_primitives::transaction::Transaction,
}

impl From<CliSignTransactionSecretKey> for SignTransactionSecretKey {
    fn from(item: CliSignTransactionSecretKey) -> Self {
        let signer_secret_key: near_crypto::SecretKey = match item.signer_secret_key {
            Some(cli_signer_secret_key) => cli_signer_secret_key,
            None => SignTransactionSecretKey::input_signer_private_key(),
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
    pub fn input_signer_private_key() -> near_crypto::SecretKey {
        Input::new()
            .with_prompt("Enter the private key")
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

    pub async fn process(self) -> crate::CliResult {
        let signature = self
            .signer_secret_key
            .sign(&self.unsigned_transaction.get_hash_and_size().0.as_ref());
        let signed_transaction = near_primitives::transaction::SignedTransaction::new(
            signature,
            self.unsigned_transaction,
        );
        let serialize_to_base64 = near_primitives::serialize::to_base64(
            signed_transaction.try_to_vec().map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Transaction is not expected to fail on serialization: {}",
                    err
                ))
            })?,
        );
        println!("\n\nThe transaction has been successfully signed.");
        println!("Signed transaction:");
        crate::common::print_transaction(signed_transaction.transaction.clone());
        println!("{:<13} {}", "signature:", signed_transaction.signature);
        println!(
            "Base64-encoded signed transaction:\n{}",
            serialize_to_base64
        );
        Ok(())
    }
}
