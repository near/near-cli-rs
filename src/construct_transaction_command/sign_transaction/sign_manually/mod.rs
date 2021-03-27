use near_primitives::borsh::BorshSerialize;
use structopt::StructOpt;

#[derive(Debug)]
pub struct SignManually {}

#[derive(Debug, Default, StructOpt)]
pub struct CliSignManually {}

impl From<CliSignManually> for SignManually {
    fn from(_: CliSignManually) -> Self {
        SignManually {}
    }
}

impl SignManually {
    pub fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        _selected_server_url: Option<url::Url>,
    ) {
        println!();
        println!(
            "SignManually process: prepopulated_unsigned_transaction:\n {:#?}",
            &prepopulated_unsigned_transaction
        );
        println!();
        let serialize_to_base64 = near_primitives::serialize::to_base64(
            prepopulated_unsigned_transaction
                .try_to_vec()
                .expect("Transaction is not expected to fail on serialization"),
        );
        println!(
            "---  serialize_to_base64:   --- \n   {:#?}",
            &serialize_to_base64
        )
    }
}
