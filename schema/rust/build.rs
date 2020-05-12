use std::path::Path;

fn main() {
    let schema_files = [
        Path::new("../world.fbs"),
        Path::new("../messages.fbs"),
        Path::new("../actions.fbs"),
        Path::new("../math.fbs"),
    ];

    println!("cargo:rerun-if-changed=schema");
    flatc_rust::run(flatc_rust::Args {
        lang: "rust",
        inputs: &schema_files,
        out_dir: Path::new("src"),
        ..flatc_rust::Args::default()
    })
    .expect("flatc failed to compile the flatbuffers for rust");

    flatc_rust::run(flatc_rust::Args {
        lang: "ts",
        inputs: &schema_files,
        out_dir: Path::new("../typescript/"),
        ..flatc_rust::Args::default()
    })
    .expect("flatc failed to compile the flatbuffers for typescript");
}
