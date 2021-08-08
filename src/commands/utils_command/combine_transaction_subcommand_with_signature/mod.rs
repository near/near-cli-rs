use dialoguer::Input;
use near_primitives::borsh::BorshSerialize;

/// утилита, соединяющая подготовленную неподписанную транзакцию с синатурой
#[derive(Debug, Default, Clone, clap::Clap)]
pub struct CliCombineTransactionSignature {
    #[clap(long)]
    signature: Option<near_crypto::Signature>,
    #[clap(long)]
    unsigned_transaction: Option<crate::common::TransactionAsBase64>,
}

#[derive(Debug, Clone)]
pub struct CombineTransactionSignature {
    signature: near_crypto::Signature,
    unsigned_transaction: near_primitives::transaction::Transaction,
}

impl std::fmt::Display for CombineTransactionSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CombineTransactionSignature {}", &self)
    }
}

impl CliCombineTransactionSignature {
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
        if let Some(signature) = &self.signature {
            args.push_front(signature.to_string());
            args.push_front("--signature".to_string());
        }
        args
    }
}

impl From<CombineTransactionSignature> for CliCombineTransactionSignature {
    fn from(combine_transaction_signature: CombineTransactionSignature) -> Self {
        Self {
            signature: Some(combine_transaction_signature.signature),
            unsigned_transaction: Some(crate::common::TransactionAsBase64 {
                inner: combine_transaction_signature.unsigned_transaction,
            }),
        }
    }
}

impl From<CliCombineTransactionSignature> for CombineTransactionSignature {
    fn from(item: CliCombineTransactionSignature) -> Self {
        let signature: near_crypto::Signature = match item.signature {
            Some(cli_signature) => cli_signature,
            None => CombineTransactionSignature::input_signature(),
        };
        let unsigned_transaction: near_primitives::transaction::Transaction =
            match item.unsigned_transaction {
                Some(cli_unsigned_transaction) => cli_unsigned_transaction.inner,
                None => CombineTransactionSignature::input_unsigned_transaction(),
            };
        Self {
            signature,
            unsigned_transaction,
        }
    }
}

impl CombineTransactionSignature {
    pub fn input_signature() -> near_crypto::Signature {
        Input::new()
            .with_prompt("Enter the signature")
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
        let signed_transaction = near_primitives::transaction::SignedTransaction::new(
            self.signature,
            self.unsigned_transaction,
        );
        let serialize_to_base64 = near_primitives::serialize::to_base64(
            signed_transaction
                .try_to_vec()
                .expect("Transaction is not expected to fail on serialization"),
        );
        println!("\nThe transaction has been successfully signed.");
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
