use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct Deposit {
    #[interactive_clap(skip_default_input_arg)]
    ///Enter deposit for a function call
    deposit: crate::common::NearBalance,
    #[interactive_clap(subcommand)]
    access_key_mode: super::add_key::AccessKeyMode,
}

impl Deposit {
    fn input_deposit(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<crate::common::NearBalance> {
        println!();
        let deposit: crate::common::NearBalance = Input::new()
            .with_prompt(
                "Enter deposit for a function call (example: 10NEAR or 0.5near or 10000yoctonear).",
            )
            .with_initial_text("0.1 NEAR")
            .interact_text()?;
        Ok(deposit)
    }

    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: super::AccountProperties,
    ) -> crate::CliResult {
        let account_properties = super::AccountProperties {
            deposit: self.deposit.clone(),
            ..account_properties
        };
        self.access_key_mode
            .process(config, account_properties)
            .await
    }
}
