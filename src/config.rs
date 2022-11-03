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

    /// Sort entries by depth
    #[clap(long)]
    pub sort_by_depth: bool,

    /// Only show entries matching this regex
    #[clap(long)]
    pub filter: Option<String>,

    /// Show files as well as directories
    #[arg(long)]
    pub show_files: bool,
}

pub fn parse_args() -> FastFindConfig {
    FastFindConfig::parse()
}
