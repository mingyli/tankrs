#[path = "../../schema/schema.rs"]
mod schema;
use schema::world_generated::{get_root_as_world, World, WorldArgs};

impl World<'_> {
    // Demo custom methods.
    fn foo(&self) -> u16 {
        self.width() * self.height()
    }
}

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
    println!("foo: {}", world.foo());
}
