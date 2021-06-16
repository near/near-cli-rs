use dialoguer::Input;
use near_primitives::borsh::BorshDeserialize;

/// Using this utility, you can view the contents of a serialized transaction (signed or not).
#[derive(Debug, Default, clap::Clap)]
pub struct CliViewSerializedTransaction {
    hash: Option<String>
}

#[derive(Debug)]
pub struct ViewSerializedTransaction {
    hash: String
}

impl From<CliViewSerializedTransaction> for ViewSerializedTransaction {
    fn from(item: CliViewSerializedTransaction) -> Self {
        let hash: String = match item.hash {
            Some(hash) => hash,
            None => ViewSerializedTransaction::input_hash()
        };
        Self {hash}
    }
}

impl ViewSerializedTransaction {
    fn input_hash() -> String {
        Input::new()
            .with_prompt("Enter the hash of the transaction")
            .interact_text()
            .unwrap()
    }

    pub async fn process(self) -> crate::CliResult {
        let serialize_from_base64 = near_primitives::serialize::from_base64(&self.hash).unwrap();
        match near_primitives::transaction::Transaction::try_from_slice(&serialize_from_base64) {
            Ok(transaction) => println!("\n{:#?}", &transaction),
            Err(_) => {
                match near_primitives::transaction::SignedTransaction::try_from_slice(&serialize_from_base64) {
                    Ok(signed_transaction) => println!("\n{:#?}", signed_transaction),
                    Err(err) => println!("\nError: Base64 transaction sequence is invalid: {}", err)
                }
            }
        };
        Ok(())
    }
}
