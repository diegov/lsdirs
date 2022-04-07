use std::{path::PathBuf, str::FromStr};

use ignore::{DirEntry, WalkBuilder};

mod config;

type Result2<T> = Result<T, Box<dyn std::error::Error>>;

fn is_dir(v: &DirEntry) -> bool {
    if let Some(typ) = v.file_type() {
        typ.is_dir()
    } else {
        false
    }
}

fn main() -> Result2<()> {
    let conf = config::parse_args();

    if let Some(max_entries) = conf.max_entries {
        let mut count = 0;
        let stop_check = || {
            count += 1;
            count >= max_entries
        };

        walk(&conf, stop_check)
    } else {
        let stop_check = || false;
        walk(&conf, stop_check)
    }
}

fn walk<F>(conf: &config::FastFindConfig, mut stop_check: F) -> Result2<()>
where
    F: FnMut() -> bool,
{
    let path = PathBuf::from_str(&conf.path)?;

    for result in WalkBuilder::new(path)
        .max_depth(conf.max_depth)
        .standard_filters(true)
        .filter_entry(is_dir)
        .build()
    {
        match result {
            Ok(entry) => {
                println!("{}", entry.path().display())
            }
            Err(err) => println!("ERROR: {}", err),
        }

        if stop_check() {
            return Ok(());
        }
    }

    Ok(())
}
