use flatbuffers::FlatBufferBuilder;
use flatbuffers::{ForwardsUOffset, Vector, WIPOffset};
use schema::world_generated::{BlockBuf, BlockBufArgs, WorldBuf, WorldBufArgs};

pub use schema::world_generated::BlockType;

#[derive(Debug)]
pub struct DiscretePos {
    x: u16,
    y: u16,
}

impl DiscretePos {
    pub fn new(x: u16, y: u16) -> DiscretePos {
        DiscretePos { x, y }
    }
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

    pub fn add_to_fb<'a>(&self, builder: &mut FlatBufferBuilder<'a>) -> WIPOffset<BlockBuf<'a>> {
        BlockBuf::create(
            builder,
            &BlockBufArgs {
                x: self.position.x,
                y: self.position.y,
                block_type: self.block_type,
            },
        )
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

    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn add_world_to_fb<'a>(
        &self,
        builder: &mut FlatBufferBuilder<'a>,
    ) -> WIPOffset<WorldBuf<'a>> {
        WorldBuf::create(
            builder,
            &WorldBufArgs {
                width: self.width,
                height: self.height,
            },
        )
    }

    pub fn add_blocks_to_fb<'a>(
        &self,
        builder: &mut FlatBufferBuilder<'a>,
    ) -> WIPOffset<Vector<'a, ForwardsUOffset<BlockBuf<'a>>>> {
        let mut vec = Vec::new();

        for block in &self.blocks {
            vec.push(block.add_to_fb(builder));
        }

        builder.create_vector(vec.as_slice())
    }
}
