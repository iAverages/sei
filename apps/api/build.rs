use std::{env, fs, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let source_dir = manifest_dir.join("../../packages/prisma/prisma/migrations");
    let output_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let migrations_dir = output_dir.join("migrations");

    if migrations_dir.exists() {
        fs::remove_dir_all(&migrations_dir).unwrap();
    }
    fs::create_dir_all(&migrations_dir).unwrap();

    for entry in fs::read_dir(&source_dir).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }

        let migration = entry.path().join("migration.sql");
        if migration.exists() {
            let destination = migrations_dir.join(entry.file_name()).with_extension("sql");
            fs::copy(migration, destination).unwrap();
        }
    }

    let workspace_dir = manifest_dir.join("../..").canonicalize().unwrap();
    let relative_migrations = PathBuf::from("../..").join(
        migrations_dir
            .strip_prefix(workspace_dir)
            .expect("Cargo target directory must be inside the workspace"),
    );
    let generated = format!(
        "static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!({:?});\n",
        relative_migrations
    );
    fs::write(output_dir.join("migrations.rs"), generated).unwrap();

    println!("cargo:rerun-if-changed={}", source_dir.display());
}
