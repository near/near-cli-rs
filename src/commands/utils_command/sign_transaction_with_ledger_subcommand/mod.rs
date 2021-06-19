use dialoguer::Input;
use near_primitives::borsh::BorshSerialize;
use std::str::FromStr;

/// Utility to sign transaction on Ledger
#[derive(Debug, Default, clap::Clap)]
pub struct CliSignTransactionWithLedger {
    #[clap(long)]
    seed_phrase_hd_path: Option<slip10::BIP32Path>,
    #[clap(long)]
    unsigned_transaction: Option<crate::common::TransactionAsBase64>,
}

#[derive(Debug)]
pub struct SignTransactionWithLedger {
    pub seed_phrase_hd_path: slip10::BIP32Path,
    pub unsigned_transaction: near_primitives::transaction::Transaction,
}

impl From<CliSignTransactionWithLedger> for SignTransactionWithLedger {
    fn from(item: CliSignTransactionWithLedger) -> Self {
        let seed_phrase_hd_path = match item.seed_phrase_hd_path {
            Some(hd_path) => hd_path,
            None => slip10::BIP32Path::from_str("44'/397'/0'/0'/1'").unwrap(),
        };
        let unsigned_transaction: near_primitives::transaction::Transaction =
            match item.unsigned_transaction {
                Some(cli_unsigned_transaction) => cli_unsigned_transaction.inner,
                None => SignTransactionWithLedger::input_unsigned_transaction(),
            };
        SignTransactionWithLedger {
            seed_phrase_hd_path,
            unsigned_transaction,
        }
    }
}

impl SignTransactionWithLedger {
    pub fn input_unsigned_transaction() -> near_primitives::transaction::Transaction {
        let input: crate::common::TransactionAsBase64 = Input::new()
            .with_prompt("Enter an unsigned transaction")
            .interact_text()
            .unwrap();
        input.inner
    }

    pub async fn process(self) -> crate::CliResult {
        println!(
            "Going to sign transaction:\n{:#?}",
            self.unsigned_transaction
        );

        println!(
            "Please confirm transaction signing on Ledger Device (HD Path {})",
            self.seed_phrase_hd_path.to_string()
        );
        let signature = match near_ledger::sign_transaction(
            self.unsigned_transaction
                .try_to_vec()
                .expect("Transaction is not expected to fail on serialization"),
            self.seed_phrase_hd_path,
        )
        .await
        {
            Ok(signature) => {
                near_crypto::Signature::from_parts(near_crypto::KeyType::ED25519, &signature)
                    .expect("Signature is not expected to fail on deserialization")
            }
            Err(near_ledger_error) => {
                return Err(color_eyre::Report::msg(format!(
                    "Error occurred while signing the transaction: {:?}",
                    near_ledger_error
                )));
            }
        };

        let signed_transaction = near_primitives::transaction::SignedTransaction::new(
            signature,
            self.unsigned_transaction,
        );

        println!("Signed transaction:\n{:#?}", signed_transaction);

        let serialize_to_base64 = near_primitives::serialize::to_base64(
            signed_transaction
                .try_to_vec()
                .expect("Signed transaction is not expected to fail on serialization"),
        );
        println!("Base64-encoded signed transaction: {}", serialize_to_base64);
        Ok(())
    }
}
