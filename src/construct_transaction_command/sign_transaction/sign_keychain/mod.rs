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



#[derive(Debug)]
pub struct SignKeychain {
    pub key_chain: String,
}

#[derive(Debug, StructOpt)]
pub struct CliSignKeychain {
    #[structopt(long)]
    key_chain: Option<String>,
}

impl SignKeychain {
    pub fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: String,
    ) {
        println!("SignKeychain process: self:       {:?}", &self);
        println!("SignKeychain process: prepopulated_unsigned_transaction:       {:?}", &prepopulated_unsigned_transaction);
        
    }

    pub fn input_key_chain() -> String {
        Input::new()
            .with_prompt("Enter the key chain")
            .interact_text()
            .unwrap()
    }
}

impl From<CliSignKeychain> for SignKeychain {
    fn from(item: CliSignKeychain) -> Self {
        println!("***********.  cli sign alternative {:?}", &item);
        let key_chain: String = match item.key_chain {
            Some(cli_key_chain) => cli_key_chain,
            None => SignKeychain::input_key_chain()
        };
        SignKeychain {
            key_chain,
        }
    }
}
