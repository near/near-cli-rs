use structopt::StructOpt;
use std::str::FromStr;
use strum_macros::{
    Display,
    EnumString,
    EnumVariantNames,
};
use strum::VariantNames;
use dialoguer::{
    Select,
    Input,
    theme::ColorfulTheme,
    console::Term
};
use near_primitives::borsh::BorshSerialize;


#[derive(Debug)]
pub struct SignManually {}

#[derive(Debug, StructOpt)]
pub struct CliSignManually {}

impl SignManually {
    pub fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        _selected_server_url: url::Url,
    ) {
        println!();
        println!("SignManually process: prepopulated_unsigned_transaction:\n {:#?}", &prepopulated_unsigned_transaction);
        println!();
        let serialize_to_base64 = near_primitives::serialize::to_base64(
                    prepopulated_unsigned_transaction
                    .try_to_vec()
                    .expect("Transaction is not expected to fail on serialization"),
            );
        println!("---  serialize_to_base64:   --- \n   {:#?}", &serialize_to_base64)
    }
}
