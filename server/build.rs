extern crate protoc_rust;

fn main() {
    protoc_rust::Codegen::new()
        .out_dir("src/protos")
        .inputs(&["protos/world.proto"])
        .include("protos")
        .run()
        .expect("Failed to run protoc.");
}
