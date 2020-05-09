use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=schema/");
    flatc_rust::run(flatc_rust::Args {
        inputs: &[Path::new("schema/world.fbs")],
        out_dir: Path::new("schema/rust/src"),
        ..flatc_rust::Args::default()
    })
    .expect("flatc failed to compile the flatbuffers.");
}
