use async_recursion::async_recursion;
use dialoguer::Input;
use std::str::FromStr;
use structopt::StructOpt;

use crate::common::NearBalance;
use super::super::receiver::{CliSkipNextAction, NextAction};

#[derive(Debug)]
pub struct TransferNEARTokensAction {
    pub amount: NearBalance,
    pub next_action: Box<NextAction>,
}

impl TransferNEARTokensAction {
    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) {
        println!("TransferNEARTokens process: self:\n       {:?}", &self);
        println!(
            "TransferNEARTokens process: prepopulated_unsigned_transaction:\n       {:?}",
            &prepopulated_unsigned_transaction
        );
        let amount = match self.amount {
            NearBalance(num) => num,
        };
        let action = near_primitives::transaction::Action::Transfer(
            near_primitives::transaction::TransferAction { deposit: amount },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        match *self.next_action {
            NextAction::AddAction(select_action) => {
                select_action
                    .process(unsigned_transaction, selected_server_url)
                    .await
            }
            NextAction::Skip(skip_action) => {
                skip_action
                    .process(unsigned_transaction, selected_server_url)
                    .await
            }
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct CliTransferNEARTokensAction {
    amount: Option<NearBalance>,
    #[structopt(subcommand)]
    next_action: Option<CliSkipNextAction>,
}

// #[derive(Debug,  Clone, Default, PartialEq)]
// pub struct NearBalance(u128);

// impl std::fmt::Display for NearBalance {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "NearBalance {}", self)
//     }
// }

// impl NearBalance {
//     pub fn input_amount() -> Self {
//         Input::new()
//             .with_prompt("How many NEAR Tokens do you want to transfer? (example: 10NEAR or 0.5near or 10000yoctonear)")
//             .interact_text()
//             .unwrap()
//     }
// }

// impl FromStr for NearBalance {
//     type Err = String;
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let num = s.trim().trim_end_matches(char::is_alphabetic).trim();
//         let currency= s.trim().trim_start_matches(&num).trim().to_uppercase();
//         let number = match currency.as_str() {
//             "N" | "NEAR" => {
//                 let res_split: Vec<&str> = num.split('.').collect();
//                 match res_split.len() {
//                     2 => {
//                         let num_int_yocto = res_split[0].parse::<u128>()
//                             .map_err(|err| format!("Near Balance: {}", err))?
//                             .checked_mul(10u128.pow(24))
//                             .ok_or_else(|| "Near Balance: underflow or overflow happens")?;
//                         let len_fract = res_split[1].len() as u32;
//                         let num_fract_yocto = if len_fract <= 24 {
//                             res_split[1]
//                                 .parse::<u128>()
//                                 .map_err(|err| format!("Near Balance: {}", err))?
//                                 .checked_mul(10u128.pow(24 - res_split[1].len() as u32))
//                                 .ok_or_else(|| "Near Balance: underflow or overflow happens")?
//                         } else {
//                             return  Err("Near Balance: too large fractional part of a number".to_string())
//                         };
//                         num_int_yocto.checked_add(num_fract_yocto)
//                             .ok_or_else(|| "Near Balance: underflow or overflow happens")?
//                     },
//                     1 => {
//                         res_split[0].parse::<u128>()
//                             .map_err(|err| format!("Near Balance: {}", err))?
//                             .checked_mul(10u128.pow(24))
//                             .ok_or_else(|| "Near Balance: underflow or overflow happens")?
//                     },
//                     _ => return Err("Near Balance: incorrect number entered".to_string())
//                 }
//             },
//             "YN" | "YNEAR" | "YOCTONEAR" | "YOCTON" => {
//                 num.parse::<u128>()
//                     .map_err(|err| format!("Near Balance: {}", err))?
//             },
//             _ => return Err("Near Balance: incorrect currency value entered".to_string())
//         };
//         Ok(NearBalance(number))
//     }
// }

impl From<CliTransferNEARTokensAction> for TransferNEARTokensAction {
    fn from(item: CliTransferNEARTokensAction) -> Self {
        let amount: NearBalance = match item.amount {
            Some(cli_amount) => cli_amount,
            None => NearBalance::input_amount(),
        };
        let next_action: Box<NextAction> = match item.next_action {
            Some(cli_skip_action) => Box::new(NextAction::from(cli_skip_action)),
            None => Box::new(NextAction::input_next_action()),
        };
        TransferNEARTokensAction {
            amount,
            next_action,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn near_balance_from_str_currency_near() {
        assert_eq!(NearBalance::from_str("10 near").unwrap(), NearBalance(10000000000000000000000000)); // 26 number
        assert_eq!(NearBalance::from_str("10.055NEAR").unwrap(), NearBalance(10055000000000000000000000)); // 26 number
    }
    #[test]
    fn near_balance_from_str_currency_n() {
        assert_eq!(NearBalance::from_str("10 n").unwrap(), NearBalance(10000000000000000000000000)); // 26 number
        assert_eq!(NearBalance::from_str("10N ").unwrap(), NearBalance(10000000000000000000000000)); // 26 number
    }
    #[test]
    fn near_balance_from_str_f64_near() {
        assert_eq!(NearBalance::from_str("0.000001 near").unwrap(), NearBalance(1000000000000000000)); // 18 number
    }
    #[test]
    fn near_balance_from_str_f64_near_without_int() {
        let near_balance = NearBalance::from_str(".055NEAR");
        assert_eq!(near_balance, Err("Near Balance: cannot parse integer from empty string".to_string()));
    }
    #[test]
    fn near_balance_from_str_currency_ynear() {
        assert_eq!(NearBalance::from_str("100 ynear").unwrap(), NearBalance(100));
        assert_eq!(NearBalance::from_str("100YNEAR ").unwrap(), NearBalance(100));
    }
    #[test]
    fn near_balance_from_str_currency_yn() {
        assert_eq!(NearBalance::from_str("9000 YN  ").unwrap(), NearBalance(9000));
        assert_eq!(NearBalance::from_str("0 yn").unwrap(), NearBalance(0));
    }
    #[test]
    fn near_balance_from_str_currency_yoctonear() {
        assert_eq!(NearBalance::from_str("111YOCTONEAR").unwrap(), NearBalance(111));
        assert_eq!(NearBalance::from_str("333 yoctonear").unwrap(), NearBalance(333));
    }
    #[test]
    fn near_balance_from_str_currency_yocton() {
        assert_eq!(NearBalance::from_str("10YOCTON").unwrap(), NearBalance(10));
        assert_eq!(NearBalance::from_str("10 yocton      ").unwrap(), NearBalance(10));
    }
    #[test]
    fn near_balance_from_str_f64_ynear() {
        let near_balance = NearBalance::from_str("0.055yNEAR");
        assert_eq!(near_balance, Err("Near Balance: invalid digit found in string".to_string()));
    }
    #[test]
    fn near_balance_from_str_without_currency() {
        let near_balance = NearBalance::from_str("100");
        assert_eq!(near_balance, Err("Near Balance: incorrect currency value entered".to_string()));
    }
    #[test]
    fn near_balance_from_str_incorrect_currency() {
        let near_balance = NearBalance::from_str("100 UAH");
        assert_eq!(near_balance, Err("Near Balance: incorrect currency value entered".to_string()));
    }
    #[test]
    fn near_balance_from_str_invalid_double_dot() {
        let near_balance = NearBalance::from_str("100.55.");
        assert_eq!(near_balance, Err("Near Balance: incorrect currency value entered".to_string()));
    }
    #[test]
    fn near_balance_from_str_large_fractional_part() {
        let near_balance = NearBalance::from_str("100.1111122222333334444455555 n"); // 25 symbols after "."
        assert_eq!(near_balance, Err("Near Balance: too large fractional part of a number".to_string()));
    }
    #[test]
    fn near_balance_from_str_large_int_part() {
        let near_balance = NearBalance::from_str("1234567890123456.0 n");
        assert_eq!(near_balance, Err("Near Balance: underflow or overflow happens".to_string()));
    }
    #[test]
    fn near_balance_from_str_without_fractional_part() {
        let near_balance = NearBalance::from_str("100. n");
        assert_eq!(near_balance, Err("Near Balance: cannot parse integer from empty string".to_string()));
    }
}
