use dialoguer::Input;
use near_primitives::borsh::BorshDeserialize;

/// Using this utility, you can view the contents of a serialized transaction (signed or not).
#[derive(Debug, Default, Clone, clap::Clap)]
pub struct CliViewSerializedTransaction {
    transaction: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ViewSerializedTransaction {
    transaction: String,
}

impl CliViewSerializedTransaction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = std::collections::VecDeque::new();
        if let Some(transaction) = &self.transaction {
            args.push_front(transaction.to_string());
        }
        args
    }
}

impl From<ViewSerializedTransaction> for CliViewSerializedTransaction {
    fn from(view_serialized_transaction: ViewSerializedTransaction) -> Self {
        Self {
            transaction: Some(view_serialized_transaction.transaction),
        }
    }
}

impl From<CliViewSerializedTransaction> for ViewSerializedTransaction {
    fn from(item: CliViewSerializedTransaction) -> Self {
        let transaction: String = match item.transaction {
            Some(transaction) => transaction,
            None => ViewSerializedTransaction::input_transaction(),
        };
        Self { transaction }
    }
}

impl ViewSerializedTransaction {
    fn input_transaction() -> String {
        Input::new()
            .with_prompt("Enter the hash of the transaction")
            .interact_text()
            .unwrap()
    }

    pub async fn process(self) -> crate::CliResult {
        let serialize_from_base64 =
            near_primitives::serialize::from_base64(&self.transaction).unwrap();
        match near_primitives::transaction::Transaction::try_from_slice(&serialize_from_base64) {
            Ok(transaction) => println!("\n{:#?}", &transaction),
            Err(_) => {
                match near_primitives::transaction::SignedTransaction::try_from_slice(
                    &serialize_from_base64,
                ) {
                    Ok(signed_transaction) => {
                        println!("\nSigned transaction:\n");
                        crate::common::print_transaction(signed_transaction.transaction.clone());
                        println!("{:<13} {}", "signature:", signed_transaction.signature)
                    }
                    Err(err) => {
                        println!("\nError: Base64 transaction sequence is invalid: {}", err)
                    }
                }
            }
        };
        Ok(())
    }
}
