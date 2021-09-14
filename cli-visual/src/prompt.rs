use dialoguer::{console::Term, theme::ColorfulTheme, Input, Select};
use strum::{IntoEnumIterator, EnumMessage};

use std::str::FromStr;
use std::fmt::{Debug, Display};

pub trait PromptMessage {
    const MSG: &'static str;
}

pub trait PromptInput {
    fn prompt_input() -> Self;
}

impl<T> PromptInput for T
where
    T: PromptMessage,
    T: Clone + FromStr + Display,
    T::Err: Display + Debug,
{
    fn prompt_input() -> Self {
        prompt_input()
    }
}
pub trait Interactive<T> {
    fn interactive(self) -> T;
}

impl<T> Interactive<Self> for Option<T>
where
    T: PromptInput + Interactive<T>,
{
    fn interactive(self) -> Self {
        Some(self.unwrap_or_else(T::prompt_input).interactive())
    }
}

pub fn prompt_input_loop<T>(prompt_msg: String, valid: impl Fn(T) -> bool) -> T
where
    T: Copy + Clone + FromStr + Display + Debug,
    <T as FromStr>::Err: Display + Debug,
{
    loop {
        let val: T = prompt_input_with_msg(prompt_msg.clone());
        if valid(val) {
            break val;
        }
    }
}

pub fn prompt_input<T>() -> T
where
    T: PromptMessage + Clone + FromStr + Display,
    T::Err: Display + Debug,
{
    prompt_input_with_msg(T::MSG)
}

pub fn prompt_input_with_msg<T>(prompt_msg: impl Into<String>) -> T
where
    T: Clone + FromStr + Display,
    T::Err: Display + Debug,
{
    Input::new()
        .with_prompt(prompt_msg)
        .interact_text()
        .unwrap()
}

pub fn prompt_variant_vec(choices: Vec<&str>, prompt_msg: &str) -> Option<usize> {
    Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt_msg)
        .items(&choices)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .unwrap()
}

pub fn prompt_variant<T>(prompt: &str) -> T
where
    T: IntoEnumIterator + EnumMessage,
    T: Copy + Clone,
{
    let variants = T::iter().collect::<Vec<_>>();
    let actions = variants
        .iter()
        .map(|p| p.get_message().unwrap().to_owned())
        .collect::<Vec<_>>();

    let selected = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&actions)
        .default(0)
        .interact()
        .unwrap();

    variants[selected]
}
