// use structopt::StructOpt;
// use std::str::FromStr;
// use strum_macros::{
//     Display,
//     EnumString,
//     EnumVariantNames,
// };
// use strum::VariantNames;
// use dialoguer::{
//     Select,
//     Input,
//     theme::ColorfulTheme,
//     console::Term
// };
// use super::super::{
//     ActionSubcommand,
// };

// #[derive(Debug, StructOpt)]
// pub struct CallFunction {
//     method_name: String,
//     args: String,
//     gas: u64,  // default 1000000000
//     deposit: u128,  // default 0
//     #[structopt(subcommand)]
//     next_action: Box<ActionSubcommand>
// }
