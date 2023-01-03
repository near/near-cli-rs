#[derive(Debug, Clone, clap::Parser)]
pub struct JsArgs {
    #[clap(num_args = 0..)]
    js_args: Vec<String>,
}
