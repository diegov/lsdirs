use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

fn main() {
    let src_dir = PathBuf::from("migrations");

    print!(
        "cargo:rerun-if-changed={}",
        src_dir.as_os_str().to_str().unwrap()
    );

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let project_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    let output_path = out_dir.join("migrations.rs");

    let file = File::create(&output_path).unwrap();
    let mut writer = BufWriter::new(file);

    write!(writer, "pub const FILES: &[(u32, &str, &[u8])] = &[").unwrap();

    let mut first = true;
    for (id, name, path) in get_migrations(&src_dir) {
        println!(
            "cargo:rerun-if-changed={}",
            path.as_os_str().to_str().unwrap()
        );

        if !first {
            write!(writer, ",").unwrap();
        }

        let full_path =
            pathdiff::diff_paths(project_dir.join(path.clone()), out_dir.clone()).unwrap();

        write!(
            writer,
            "\n  ({}, \"{}\", include_bytes!(\"{}\"))",
            id,
            &name,
            &full_path.as_os_str().to_str().unwrap()
        )
        .unwrap();

        first = false;
    }
    writeln!(writer, "\n];").unwrap();
}

fn get_migrations(src_dir: &PathBuf) -> Vec<(u32, String, PathBuf)> {
    let mut result = vec![];

    for entry in fs::read_dir(src_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() && path.extension().unwrap_or_default() == "sql" {
            let name = path.file_name().unwrap().to_str().unwrap();
            let id = name.split('_').next().unwrap().parse::<u32>().unwrap();
            result.push((id, name.to_string(), path));
        }
    }

    result.sort();
    result
}
