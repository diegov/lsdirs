use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
    str::FromStr,
};

use ignore::{DirEntry, WalkBuilder};

mod config;

type LsDirsResult<T> = Result<T, Box<dyn std::error::Error>>;
type IgnoreResult = Result<ignore::DirEntry, ignore::Error>;

fn is_dir(v: &DirEntry) -> bool {
    if let Some(typ) = v.file_type() {
        // TODO: typ.is_symlink() && is_link_to_dir(&v)
        typ.is_dir()
    } else {
        false
    }
}

fn any(_: &DirEntry) -> bool {
    true
}

fn main() -> LsDirsResult<()> {
    let conf = config::parse_args();

    if let Some(max_entries) = conf.max_entries {
        let mut count = 0;
        let stop_check = || {
            count += 1;
            count >= max_entries
        };

        walk(&conf, stop_check, conf.show_files)
    } else {
        let stop_check = || false;
        walk(&conf, stop_check, conf.show_files)
    }
}

fn walk<STOP>(conf: &config::FastFindConfig, stop_check: STOP, show_files: bool) -> LsDirsResult<()>
where
    STOP: FnMut() -> bool,
{
    if show_files {
        walk_with_filter(conf, stop_check, any)
    } else {
        walk_with_filter(conf, stop_check, is_dir)
    }
}

fn walk_with_filter<STOP, FILTER>(
    conf: &config::FastFindConfig,
    stop_check: STOP,
    filter: FILTER,
) -> LsDirsResult<()>
where
    STOP: FnMut() -> bool,
    FILTER: Fn(&DirEntry) -> bool + Send + Sync + 'static,
{
    let path = PathBuf::from_str(&conf.path)?;

    // TODO: What do to about errors?
    let iter = WalkBuilder::new(path)
        .max_depth(conf.max_depth)
        .standard_filters(true)
        .filter_entry(filter)
        .build()
        .filter(|res| {
            if let Ok(d) = res {
                // TODO: use different closure instead of running
                // this comparison for each entry
                if let Some(pattern) = &conf.filter {
                    d.path().to_string_lossy().contains(pattern)
                } else {
                    true
                }
            } else {
                true
            }
        });

    if conf.sort_by_depth {
        let mut dirs: Vec<_> = iter.collect();
        dirs.sort_by(compare_by_depth);
        print_results(dirs, stop_check)?;
    } else {
        print_results(iter, stop_check)?;
    }

    Ok(())
}

fn print_results<F, IT>(results: IT, mut stop_check: F) -> LsDirsResult<()>
where
    IT: IntoIterator<Item = IgnoreResult>,
    F: FnMut() -> bool,
{
    for result in results {
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

// TODO: Include original walk depth and index for errors, to inline error
// messages among related paths as it's done in depth first traversal.
fn compare_by_depth(a: &IgnoreResult, b: &IgnoreResult) -> Ordering {
    get_depth_sort_tuple(a).cmp(&get_depth_sort_tuple(b))
}

fn get_depth_sort_tuple(v: &IgnoreResult) -> (u8, usize, Option<&Path>) {
    // Errors at the end for now, until we add depth and index to them
    match v {
        Ok(entry) => (0, entry.depth(), Some(entry.path())),
        Err(_) => (1, 0, None),
    }
}
