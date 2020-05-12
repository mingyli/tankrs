use super::serialize::Serializable;
use schema::world_generated;

pub struct World {
    width: u16,
    height: u16,
}

impl World {
    pub fn new(width: u16, height: u16) -> World {
        World { width, height }
    }
}

impl Serializable for World {
    fn serialize(&self) -> Vec<u8> {
        let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
        let world = world_generated::World::create(
            &mut builder,
            &world_generated::WorldArgs {
                width: self.width,
                height: self.height,
            },
        );
        builder.finish(world, None);
        builder.finished_data().to_vec()
    }
}
