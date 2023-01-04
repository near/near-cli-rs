#[derive(Debug, Clone, clap::Parser)]
pub struct JsArgs {
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    js_args: Vec<String>,
}
