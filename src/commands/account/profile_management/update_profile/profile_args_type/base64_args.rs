use inquire::Text;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::UpdateAccountProfileContext)]
#[interactive_clap(output_context = Base64ArgsContext)]
pub struct Base64Args {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter valid Base64-encoded string (e.g. e30=):
    data: String,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: super::super::sign_as::Signer,
}

#[derive(Clone)]
pub struct Base64ArgsContext(super::ArgsContext);

impl Base64ArgsContext {
    pub fn from_previous_context(
        previous_context: super::super::UpdateAccountProfileContext,
        scope: &<Base64Args as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(super::ArgsContext {
            global_context: previous_context.global_context,
            get_contract_account_id: previous_context.get_contract_account_id,
            account_id: previous_context.account_id,
            data: near_primitives::serialize::from_base64(&scope.data)
                .map_err(|_| color_eyre::eyre::eyre!("Data cannot be decoded with base64"))?,
        }))
    }
}

impl From<Base64ArgsContext> for super::ArgsContext {
    fn from(item: Base64ArgsContext) -> Self {
        item.0
    }
}

impl Base64Args {
    fn input_data(
        _context: &super::super::UpdateAccountProfileContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        loop {
            let data = Text::new("Enter valid Base64-encoded string (e.g. e30=):").prompt()?;
            if near_primitives::serialize::from_base64(&data).is_ok() {
                return Ok(Some(data));
            }
            eprintln!("Data not in Base64 format!")
        }
    }
}
