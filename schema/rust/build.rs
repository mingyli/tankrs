use glob::glob;
use std::path::{Path, PathBuf};

fn main() {
    let path_bufs: Vec<PathBuf> = glob("../*.fbs")
        .expect("failed to glob flatbuffer defs")
        .map(|x| x.unwrap())
        .collect();
    let paths: Vec<&Path> = path_bufs.iter().map(|x| x.as_path()).collect();

    println!("cargo:rerun-if-changed=schema");
    flatc_rust::run(flatc_rust::Args {
        inputs: paths.as_slice(),
        out_dir: Path::new("src"),
        ..flatc_rust::Args::default()
    })
    .expect("flatc failed to compile the flatbuffers.");
}
