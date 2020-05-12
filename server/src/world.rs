use super::serialize::Serializable;
use schema::world_generated;
pub struct Block {
    x: f32,
}
pub struct World {
    width: u16,
    height: u16,
    block: Block,
}

impl World {
    pub fn new(width: u16, height: u16) -> World {
        World {
            width,
            height,
            block: Block { x: 0.0 },
        }
    }
}

impl Serializable for World {
    fn serialize(&self) -> Vec<u8> {
        let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
        let block = world_generated::Block::create(
            &mut builder,
            &world_generated::BlockArgs {
                x: self.block.x,
                v: Some(&world_generated::MyVec::new(1.0, 2.0)),
            },
        );
        let world = world_generated::World::create(
            &mut builder,
            &world_generated::WorldArgs {
                width: self.width,
                height: self.height,
                block: Some(block),
            },
        );
        let message = world_generated::Message::create(
            &mut builder,
            &world_generated::MessageArgs {
                thing_type: world_generated::Thing::World,
                thing: Some(world.as_union_value()),
            }
        );
        builder.finish(message, None);
        builder.finished_data().to_vec()
    }
}
