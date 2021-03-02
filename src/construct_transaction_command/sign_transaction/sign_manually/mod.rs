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
pub struct SignManually {

}

#[derive(Debug, StructOpt)]
pub struct CliSignManually {
    
}
