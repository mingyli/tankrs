#[allow(dead_code, unused_imports, clippy::redundant_field_names)]
#[path = "../../schema/world_generated.rs"]
mod world_generated;
pub use world_generated::tankrs::{get_root_as_world, World, WorldArgs};

fn main() {
    let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
    {
        let world = World::create(
            &mut builder,
            &WorldArgs {
                width: 30,
                height: 20,
            },
        );
        builder.finish(world, None);
    }

    let buffer = builder.finished_data();
    let world = get_root_as_world(buffer);
    let width = world.width();
    let height = world.height();
    println!("My world has width {} and height {}.", width, height);
}
