#![allow(clippy::arc_with_non_send_sync)]
pub use common::CliResult;

use inquire::ui::{Color, RenderConfig, StyleSheet, Styled};

pub mod commands;
pub mod common;
pub mod config;
pub mod js_command_match;
pub mod network;
pub mod network_for_transaction;
pub mod network_view_at_block;
pub mod transaction_signature_options;
pub mod types;
pub mod utils_command;

#[derive(Debug, Clone)]
pub struct GlobalContext {
    pub config: crate::config::Config,
    pub offline: bool,
    pub verbosity: Verbosity,
}

#[derive(Debug, Clone, Default)]
pub enum Verbosity {
    #[default]
    Interactive,
    TeachMe,
    Quiet,
}

pub fn setup_tracing(verbosity: Verbosity) -> CliResult {
    use tracing::{Event, Level, Subscriber};
    use tracing_indicatif::style::ProgressStyle;
    use tracing_indicatif::IndicatifLayer;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::{
        fmt::{format::Writer, FmtContext, FormatEvent, FormatFields},
        registry::LookupSpan,
    };

    struct SimpleFormatter;

    impl<S, N> FormatEvent<S, N> for SimpleFormatter
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
        N: for<'a> FormatFields<'a> + 'static,
    {
        fn format_event(
            &self,
            ctx: &FmtContext<'_, S, N>,
            mut writer: Writer<'_>,
            event: &Event<'_>,
        ) -> std::fmt::Result {
            let level = *event.metadata().level();
            let (icon, color_code) = match level {
                Level::TRACE => ("TRACE ", "\x1b[35m"),   // Magenta
                Level::DEBUG => ("DEBUG ", "\x1b[34m"),   // Blue
                Level::INFO => ("", ""),                  // Default
                Level::WARN => ("Warning: ", "\x1b[33m"), // Yellow
                Level::ERROR => ("ERROR ", "\x1b[31m"),   // Red
            };

            write!(writer, "{}├  {}", color_code, icon)?;

            write!(writer, "\x1b[0m")?;

            ctx.field_format().format_fields(writer.by_ref(), event)?;

            writeln!(writer)
        }
    }

    match verbosity {
        Verbosity::TeachMe => {
            let env_filter = EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into())
                .add_directive("near_teach_me=info".parse()?)
                .add_directive("near_cli_rs=info".parse()?);
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().event_format(SimpleFormatter))
                .with(env_filter)
                .init();
        }
        Verbosity::Interactive => {
            let indicatif_layer = IndicatifLayer::new()
                .with_progress_style(
                    ProgressStyle::with_template(
                        "{spinner:.blue}{span_child_prefix} {span_name} {msg} {span_fields}",
                    )
                    .unwrap()
                    .tick_strings(&["◐", "◓", "◑", "◒"]),
                )
                .with_span_child_prefix_symbol("↳ ");
            let env_filter = EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into())
                .add_directive("near_cli_rs=info".parse()?);
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::fmt::layer()
                        .event_format(SimpleFormatter)
                        .with_writer(indicatif_layer.get_stderr_writer()),
                )
                .with(indicatif_layer)
                .with(env_filter)
                .init();
        }
        Verbosity::Quiet => {}
    };
    Ok(())
}

pub fn get_global_render_config() -> RenderConfig<'static> {
    let mut render_config = RenderConfig::default_colored();
    render_config.prompt_prefix = Styled::new("◆ ").with_fg(Color::DarkGreen);
    render_config.answered_prompt_prefix = Styled::new("◇ ").with_fg(Color::DarkGreen);
    render_config.highlighted_option_prefix = Styled::new(" ●").with_fg(Color::DarkGreen);
    render_config.unhighlighted_option_prefix = Styled::new(" ○").with_fg(Color::DarkGrey);
    render_config.selected_checkbox = Styled::new("◼").with_fg(Color::LightGreen);
    render_config.scroll_up_prefix = Styled::new("↑○").with_fg(Color::DarkGrey);
    render_config.scroll_down_prefix = Styled::new("↓○").with_fg(Color::DarkGrey);
    render_config.unselected_checkbox = Styled::new("◻").with_fg(Color::DarkGrey);
    render_config.option = StyleSheet::new().with_fg(Color::DarkGrey);
    render_config.selected_option = Some(StyleSheet::new().with_fg(Color::Grey));

    render_config.new_line_prefix = Some(Styled::new("│  ").with_fg(Color::LightBlue));
    render_config.answer_from_new_line = true;

    render_config.error_message = render_config
        .error_message
        .with_prefix(Styled::new("❌").with_fg(Color::LightRed));

    render_config.text_input = StyleSheet::new().with_fg(Color::LightYellow);

    render_config.answer = StyleSheet::new().with_fg(Color::DarkGrey);

    render_config.help_message = StyleSheet::new().with_fg(Color::DarkYellow);

    render_config
}
