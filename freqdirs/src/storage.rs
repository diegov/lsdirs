use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use crate::migrations;
use fs2::FileExt;
use sqlite::{Connection, State};

use std::time::{SystemTime, UNIX_EPOCH};

type StorageResult<T> = Result<T, Box<dyn std::error::Error>>;

fn epoch_time() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u32
}

fn init_migrations_table(connection: &Connection) -> StorageResult<i32> {
    let migrated = connection.prepare("select max(id) as max_version from migration");

    let max_id = match migrated {
        Ok(mut v) => {
            if v.next().is_ok() {
                v.read::<i64, _>("max_version")?
            } else {
                -1
            }
        }
        Err(_) => {
            let query = "create table migration (id integer, script_name text, timestamp integer);";
            connection.execute(query)?;
            -1
        }
    };

    Ok(max_id as i32)
}

fn apply_migrations(connection: &Connection) -> StorageResult<i32> {
    let max_applied = init_migrations_table(connection)?;
    let max_available = migrations::FILES
        .last()
        .map(|m| m.0 as i32)
        .unwrap_or_else(|| -1);

    if max_available > max_applied {
        for (id, name, content) in migrations::FILES {
            if *id as i32 > max_applied {
                println!("Running migration {}", name);
                let script = String::from_utf8(content.to_vec())?;
                connection.execute("BEGIN;")?;
                connection.execute(&script)?;
                let mut version_stms = connection.prepare(
                    "insert into migration (id, script_name, timestamp) values (?, ?, ?);",
                )?;
                let now = epoch_time();
                version_stms.bind((1, *id as i64))?;
                version_stms.bind((2, *name))?;
                version_stms.bind((3, now as i64))?;
                version_stms.next()?;
                connection.execute("COMMIT;")?;
            }
        }
    }

    Ok(max_applied)
}

const FORGET_THRESHOLD: i64 = 7 * 24 * 60 * 60 * 1000;

pub fn get_paths(
    db_dir: &Path,
    root_path: &Option<PathBuf>,
    work_dir: &Option<PathBuf>,
) -> StorageResult<Vec<String>> {
    let db = DbContext::new(db_dir)?;

    // TODO: Use max(last_seen) across all paths instead of current time as a reference
    // 1 repeat = 10 minutes for scoring
    let mut stmt = db.connection.prepare(
        r#"
select canonical_path from (
select canonical_path,
  case when canonical_path like ? || '%' then 1.0 else 0.0 end as workdir_score,
  sqrt(seen_count * 1.0) + sqrt((? - last_seen) / (10 * 60 * 1000)) as score
from freq_path
where ? - last_seen < ?
and canonical_path like ? || '%'
) q
order by workdir_score desc, score desc
"#,
    )?;
    let now = epoch_time() as i64;

    if let Some(path) = work_dir {
        stmt.bind((1, path.to_str().unwrap()))?;
    } else {
        stmt.bind((1, ""))?;
    }

    stmt.bind((2, now))?;
    stmt.bind((3, now))?;
    stmt.bind((4, FORGET_THRESHOLD))?;

    if let Some(path) = root_path {
        stmt.bind((5, path.to_str().unwrap()))?;
    } else {
        stmt.bind((5, ""))?;
    }

    let mut result = vec![];

    while let Ok(State::Row) = stmt.next() {
        result.push(stmt.read::<String, _>("canonical_path")?.to_string());
    }

    Ok(result)
}

pub fn update_path(db_dir: &Path, path: &Path) -> StorageResult<()> {
    let db = DbContext::new(db_dir)?;

    let mut stmt = db.connection.prepare(
        r#"update freq_path 
set
  last_seen = max(last_seen, ?),
  seen_count = case when ? - last_seen > ? then 1 else seen_count + 1 end
where canonical_path = ?;
"#,
    )?;

    let now = epoch_time() as i64;

    stmt.bind((1, now))?;
    stmt.bind((2, now))?;
    stmt.bind((3, FORGET_THRESHOLD))?;
    stmt.bind((4, path.to_str().unwrap()))?;

    stmt.next()?;

    Ok(())
}

pub fn save_path(db_dir: &Path, path: &Path) -> StorageResult<()> {
    let db = DbContext::new(db_dir)?;

    let mut stmt = db.connection.prepare(
        r#"insert into freq_path (canonical_path, last_seen, seen_count)
values (?, ?, 1)
on conflict (canonical_path) do update
set
  last_seen = max(last_seen, excluded.last_seen),
  seen_count = case when excluded.last_seen - last_seen > ? then 1 else seen_count + 1 end;'
"#,
    )?;

    let now = epoch_time() as i64;

    stmt.bind((1, path.to_str().unwrap()))?;
    stmt.bind((2, now))?;
    stmt.bind((3, FORGET_THRESHOLD))?;

    stmt.next()?;

    Ok(())
}

struct DbContext {
    connection: Connection,
    db_file: File,
}

impl DbContext {
    fn new(db_dir: &Path) -> Result<DbContext, Box<dyn std::error::Error>> {
        let path = db_dir.join("freqdirs.db");

        let db_file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;

        db_file.lock_exclusive()?;

        let connection = sqlite::open(path)?;

        apply_migrations(&connection)?;

        Ok(DbContext {
            connection,
            db_file,
        })
    }
}

impl Drop for DbContext {
    fn drop(&mut self) {
        self.db_file.unlock().unwrap();
    }
}

// pub fn query(sql: &str) -> StorageResult<()> {
//     let db = DbContext::new()?;
//     db.connection.execute(sql)?;
//     drop(db);
//     Ok(())
// }
