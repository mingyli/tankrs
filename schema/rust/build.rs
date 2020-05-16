use std::path::{Path, PathBuf};

use anyhow::Result;
use glob::glob;

fn main() -> Result<()> {
    let path_bufs = glob("../*.fbs")
        .expect("failed to glob flatbuffer defs")
        .collect::<Result<Vec<PathBuf>, _>>()?;

    let paths: Vec<&Path> = path_bufs.iter().map(std::path::PathBuf::as_path).collect();

    println!("cargo:rerun-if-changed=schema");
    flatc_rust::run(flatc_rust::Args {
        inputs: paths.as_slice(),
        out_dir: Path::new("src"),
        ..flatc_rust::Args::default()
    })
    .expect("flatc failed to compile the flatbuffers.");

    Ok(())
}
