#![allow(clippy::arc_with_non_send_sync)]
pub use common::CliResult;

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
    pub teach_me: bool,
}

pub fn setup_tracing(teach_me_flag_is_set: bool) -> CliResult {
    use indicatif::ProgressStyle;
    use tracing_indicatif::IndicatifLayer;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::EnvFilter;

    if teach_me_flag_is_set {
        let env_filter = EnvFilter::from_default_env()
            .add_directive(tracing::Level::WARN.into())
            .add_directive("near_teach_me=info".parse()?)
            .add_directive("near_cli_rs=info".parse()?);
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .without_time()
                    .with_target(false),
            )
            .with(env_filter)
            .init();
    } else {
        let indicatif_layer = IndicatifLayer::new()
            .with_progress_style(
                ProgressStyle::with_template(
                    "{spinner:.blue}{span_child_prefix} {span_name} {msg} {span_fields}",
                )
                .unwrap()
                .tick_strings(&[
                    "▹▹▹▹▹",
                    "▸▹▹▹▹",
                    "▹▸▹▹▹",
                    "▹▹▸▹▹",
                    "▹▹▹▸▹",
                    "▹▹▹▹▸",
                    "▪▪▪▪▪",
                ]),
            )
            .with_span_child_prefix_symbol("↳ ");
        let env_filter = EnvFilter::from_default_env()
            .add_directive(tracing::Level::WARN.into())
            .add_directive("near_cli_rs=info".parse()?);
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .without_time()
                    .with_writer(indicatif_layer.get_stderr_writer()),
            )
            .with(indicatif_layer)
            .with(env_filter)
            .init();
    };
    Ok(())
}
