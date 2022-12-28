#[derive(Debug, Clone, clap::Parser)]
pub struct CleanArgs {
    #[clap(long, aliases = ["out_dir", "outDir"], default_value = "./out")]
    out_dir: String,
}
