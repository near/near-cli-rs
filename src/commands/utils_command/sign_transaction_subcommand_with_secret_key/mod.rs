use dialoguer::Input;
use near_primitives::borsh::BorshSerialize;

/// a utility that allows you to sign a transaction with a secret key
#[derive(Debug, Default, Clone, clap::Clap)]
pub struct CliSignTransactionPrivateKey {
    #[clap(long)]
    signer_private_key: Option<near_crypto::SecretKey>,
    #[clap(long)]
    unsigned_transaction: Option<crate::common::TransactionAsBase64>,
}

#[derive(Debug, Clone)]
pub struct SignTransactionPrivateKey {
    pub signer_private_key: near_crypto::SecretKey,
    pub unsigned_transaction: near_primitives::transaction::Transaction,
}

impl CliSignTransactionPrivateKey {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = std::collections::VecDeque::new();
        if let Some(unsigned_transaction) = &self.unsigned_transaction {
            let unsigned_transaction_serialized_to_base64 = near_primitives::serialize::to_base64(
                unsigned_transaction
                    .inner
                    .try_to_vec()
                    .expect("Transaction is not expected to fail on serialization"),
            );
            args.push_front(unsigned_transaction_serialized_to_base64);
            args.push_front("--unsigned-transaction".to_string());
        }
        if let Some(signer_private_key) = &self.signer_private_key {
            args.push_front(signer_private_key.to_string());
            args.push_front("--signer-private-key".to_string());
        }
        args
    }
}

impl From<SignTransactionPrivateKey> for CliSignTransactionPrivateKey {
    fn from(sign_transaction_private_key: SignTransactionPrivateKey) -> Self {
        Self {
            signer_private_key: Some(sign_transaction_private_key.signer_private_key),
            unsigned_transaction: Some(crate::common::TransactionAsBase64 {
                inner: sign_transaction_private_key.unsigned_transaction,
            }),
        }
    }
}

impl From<CliSignTransactionPrivateKey> for SignTransactionPrivateKey {
    fn from(item: CliSignTransactionPrivateKey) -> Self {
        let signer_private_key: near_crypto::SecretKey = match item.signer_private_key {
            Some(cli_signer_private_key) => cli_signer_private_key,
            None => SignTransactionPrivateKey::input_signer_private_key(),
        };
        let unsigned_transaction: near_primitives::transaction::Transaction =
            match item.unsigned_transaction {
                Some(cli_unsigned_transaction) => cli_unsigned_transaction.inner,
                None => SignTransactionPrivateKey::input_unsigned_transaction(),
            };
        SignTransactionPrivateKey {
            signer_private_key,
            unsigned_transaction,
        }
    }
}

impl SignTransactionPrivateKey {
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
            .signer_private_key
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
