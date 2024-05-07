use std::{
    fs::canonicalize,
    path::{Path, PathBuf},
};

mod config;
mod error;
mod migrations;
mod storage;

use error::transpose;

type MainError = Box<dyn std::error::Error>;

fn main() -> Result<(), MainError> {
    let conf = config::parse_args();

    match conf.command {
        config::Command::Query { path, working_dir } => query(conf.state_dir, path, working_dir)?,
        config::Command::Save { path } => save(conf.state_dir, &path, false)?,
        config::Command::Update { path } => save(conf.state_dir, &path, true)?,
        config::Command::Delete { path } => delete(conf.state_dir, &path)?,
        config::Command::List { working_dir } => list(conf.state_dir, working_dir)?,
    }

    Ok(())
}

fn query(
    db_path: PathBuf,
    path: Option<String>,
    working_dir: Option<String>,
) -> Result<(), MainError> {
    let working_dir = transpose(working_dir.map(|p| get_canonical_path(&p, false)))?;
    let path = transpose(path.map(|p| get_canonical_path(&p, false)))?;

    for path in storage::get_paths(&db_path, &path, &working_dir)? {
        let display_path = if let Some(root_path) = &working_dir {
            make_relative(root_path, path)
        } else {
            path
        };

        if display_path.is_empty() {
            println!(".");
        } else {
            println!("{}", display_path);
        }
    }
    Ok(())
}

fn make_relative(root_path: &PathBuf, path: String) -> String {
    let child = Path::new(&path);
    match child.strip_prefix(root_path) {
        Ok(v) => v.as_os_str().to_str().unwrap().to_string(),
        Err(_) => path,
    }
}

fn get_canonical_path(path: &str, allow_non_existent: bool) -> Result<PathBuf, MainError> {
    let result = canonicalize(path);

    match result {
        Ok(canonical) => Ok(canonical),
        // TODO: Differentiate types of errors, only "does not exist" is allowed
        Err(e) => {
            if allow_non_existent {
                Ok(PathBuf::from(path))
            } else {
                Err(MainError::from(e))
            }
        }
    }
}

fn save(db_path: PathBuf, path: &str, update_only: bool) -> Result<(), MainError> {
    let canonical = get_canonical_path(path, false)?;
    if update_only {
        storage::update_path(&db_path, &canonical)?;
    } else {
        storage::save_path(&db_path, &canonical)?;
    }
    Ok(())
}

fn delete(db_path: PathBuf, path: &str) -> Result<(), MainError> {
    let canonical = get_canonical_path(path, true)?;
    storage::delete_path(&db_path, &canonical)?;
    Ok(())
}

fn list(db_path: PathBuf, working_dir: Option<String>) -> Result<(), MainError> {
    query(db_path, None, working_dir)
}
