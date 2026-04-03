use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let schema_dir = manifest_dir.join("src").join("control_plane").join("schema");

    println!("cargo:rerun-if-changed={}", schema_dir.display());

    let mut migration_paths = fs::read_dir(&schema_dir)
        .expect("read schema dir")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("sql"))
        .collect::<Vec<_>>();

    migration_paths.sort();

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let generated = out_dir.join("control_plane_migrations.rs");
    let mut file = fs::File::create(&generated).expect("create generated migrations file");

    writeln!(
        file,
        "fn generated_migrations() -> Migrations<'static> {{"
    )
    .expect("write generated header");
    writeln!(file, "    Migrations::new(vec![").expect("write migrations vec header");

    for path in migration_paths {
        writeln!(
            file,
            "        M::up(include_str!(r#\"{}\"#)),",
            normalize_for_rust(&path)
        )
        .expect("write migration entry");
    }

    writeln!(file, "    ])").expect("write migrations vec footer");
    writeln!(file, "}}").expect("write generated footer");
}

fn normalize_for_rust(path: &Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}
