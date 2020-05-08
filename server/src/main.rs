#[allow(dead_code, unused_imports, clippy::redundant_field_names)]
#[path = "../../schema/rust/world_generated.rs"]
mod world_generated;
use world_generated::tankrs::{get_root_as_world, World, WorldArgs};

impl World<'_> {
    // Return the width in pixels.
    fn width(&self) -> u16 {
        self.cell_width() * self.grid_width()
    }

    // Return the height in pixels.
    fn height(&self) -> u16 {
        self.cell_height() * self.grid_height()
    }
}

fn main() {
    let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
    {
        let world = World::create(
            &mut builder,
            &WorldArgs {
                grid_width: 40,
                grid_height: 30,
                ..Default::default()
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
