use anyhow::Result;

fn main() -> Result<()> {
    // TODO(mluogh): Implement globbing again. Sorry!
    protobuf_codegen_pure::Codegen::new()
        .out_dir("src/")
        .inputs(&[
            "../action.proto",
            "../geometry.proto",
            "../heartbeat.proto",
            "../world.proto",
            "../tank.proto",
        ])
        .include("../")
        .customize(protobuf_codegen_pure::Customize {
            serde_derive: Some(true),
            ..Default::default()
        })
        .run()?;

    Ok(())
}
