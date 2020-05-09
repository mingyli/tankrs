use schema::world_generated::{get_root_as_world, World, WorldArgs};

fn main() {
    let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
    {
        let world = World::create(
            &mut builder,
            &WorldArgs {
                width: 40,
                height: 30,
            },
        );
        builder.finish(world, None);
    }

    let buffer = builder.finished_data();
    let world = get_root_as_world(buffer);
    println!(
        "My world has width {} and height {}.",
        world.width(),
        world.height()
    );
}
