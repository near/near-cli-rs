#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::NetworkForImportAccountOutputContext)]
#[interactive_clap(output_context = PrintKeypairToTerminalContext)]
pub struct PrintKeypairToTerminal;

#[derive(Debug, Clone)]
pub struct PrintKeypairToTerminalContext;

impl PrintKeypairToTerminalContext {
    pub fn from_previous_context(
        previous_context: super::NetworkForImportAccountOutputContext,
        _scope: &<PrintKeypairToTerminal as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        // TODO: need to make a better display in general

        println!("Imported account keystore:");
        println!("    account_id:     {}", previous_context.account_id);
        println!(
            "    network:        {}",
            previous_context.chosen_network_config.network_name
        );
        println!(
            "    keychain props:\n{}",
            serde_json::to_string_pretty(&previous_context.key_store_property)?
        );

        Ok(Self)
    }
}
