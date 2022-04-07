use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct FastFindConfig {
    /// Path to walk on (optional)
    #[clap(default_value = ".")]
    pub path: String,

    /// Maximum depth to recurse
    #[clap(long)]
    pub max_depth: Option<usize>,

    /// Maximum entries to list
    #[clap(long)]
    pub max_entries: Option<u64>,
}

pub fn parse_args() -> FastFindConfig {
    FastFindConfig::parse()
}
