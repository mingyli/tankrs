mod protos;
use protos::world::World;

fn main() {
    let world = World {
        width: 30,
        height: 20,
        ..Default::default()
    };
    println!(
        "My world has width {} and height {}.",
        world.get_width(),
        world.get_height()
    );
}
