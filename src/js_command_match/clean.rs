#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `clean` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct CleanArgs {
    #[clap(long, aliases = ["out_dir", "outDir"], default_value = "./out")]
    out_dir: String,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}
