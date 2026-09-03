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
        let mut display_string = String::new();
        display_string.push_str("Imported account:");

        let mut display_info = String::new();
        display_info.push_str(&format!(
            "{:<13} {}",
            "account id:", previous_context.account_id
        ));

        display_info.push_str(&format!(
            "\n{:<13} {}",
            "network:", previous_context.chosen_network_config.network_name
        ));

        display_info.push_str(&format!(
            "\n{:<13} \n{}",
            "keychain props:",
            crate::common::indent_payload(&serde_json::to_string_pretty(
                &previous_context.key_store_property,
            )?)
        ));

        display_string.push_str(&format!(
            "\n{}",
            crate::common::indent_payload(&display_info)
        ));

        tracing::info!(
             parent: &tracing::Span::none(),
             "{}",
             display_string
        );

        Ok(Self)
    }
}
