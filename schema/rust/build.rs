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
            "../client_message.proto",
            "../server_message.proto",
        ])
        .include("../")
        .customize(protobuf_codegen_pure::Customize {
            serde_derive: Some(true),
            ..protobuf_codegen_pure::Customize::default()
        })
        .run()?;

    Ok(())
}
