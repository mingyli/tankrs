use flatbuffers::FlatBufferBuilder;
use schema::world_generated;

#[derive(Debug)]
pub struct DiscretePos(pub u16, pub u16);

#[derive(Debug)]
pub enum BlockType {
    DESTRUCTIBLE,
    INDESTRUCTIBLE,
}

#[derive(Debug)]
pub struct Block {
    // The position of the lower left corner of this block in the coordinate
    // frame.
    position: DiscretePos,
    // The type of block this represents.
    block_type: BlockType,
}

pub struct World {
    // The width and height of this world in our coordinate frame.
    width: u16,
    height: u16,

    blocks: Vec<Block>,
}

impl Block {
    pub fn new(position: DiscretePos, block_type: BlockType) -> Block {
        Block {
            position,
            block_type,
        }
    }
}

impl World {
    pub fn new(width: u16, height: u16) -> World {
        World {
            width,
            height,
            blocks: Vec::new(),
        }
    }

    pub fn to_fb_bytes(&self, builder: &mut FlatBufferBuilder) -> Vec<u8> {
        builder.reset();

        let world = world_generated::World::create(
            builder,
            &world_generated::WorldArgs {
                width: self.width,
                height: self.height,
            },
        );

        builder.finish(world, None);
        builder.finished_data().to_vec()
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }
}
