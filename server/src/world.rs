// TODO(mluogh): remove after we actually start using these
#![allow(dead_code)]
use flatbuffers::FlatBufferBuilder;
use flatbuffers::{ForwardsUOffset, Vector, WIPOffset};
pub use schema::world_generated::BlockType;
use schema::world_generated::{BlockBuf, BlockBufArgs, WorldBuf, WorldBufArgs};

#[derive(Debug)]
pub struct Position {
    x: f32,
    y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Position {
        Position { x, y }
    }
}

#[derive(Debug)]
pub struct Block {
    // The position of the lower left corner of this block in the coordinate
    // frame.
    position: Position,

    // The type of block this represents.
    block_type: BlockType,
}

#[derive(Debug)]
pub struct World {
    // The width and height of this world in our coordinate frame.
    width: u16,
    height: u16,

    blocks: Vec<Block>,
}

impl Block {
    pub fn new(x: u16, y: u16, block_type: BlockType) -> Block {
        Block {
            position: Position::new(x as f32, y as f32),
            block_type,
        }
    }

    pub fn add_to_fb<'a>(&self, builder: &mut FlatBufferBuilder<'a>) -> WIPOffset<BlockBuf<'a>> {
        BlockBuf::create(
            builder,
            &BlockBufArgs {
                x: self.position.x as u16,
                y: self.position.y as u16,
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

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
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

#[cfg(test)]
mod tests {
    use super::*;

    use schema::message_generated::{
        get_root_as_visible_state_buf, VisibleStateBuf, VisibleStateBufArgs,
    };
    use schema::world_generated::get_root_as_world_buf;

    #[test]
    fn world_can_be_serialized() {
        let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
        let my_world = World::new(69, 420);
        let world_buf = my_world.add_world_to_fb(&mut builder);
        builder.finish(world_buf, None);

        let serialized_world = get_root_as_world_buf(builder.finished_data());

        assert_eq!(serialized_world.width(), my_world.width());
        assert_eq!(serialized_world.height(), my_world.height());
    }

    #[test]
    fn blocks_can_be_serialized() {
        let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
        let mut my_world = World::new(69, 420);
        my_world.add_block(Block::new(0, 0, BlockType::Destructible));
        my_world.add_block(Block::new(0, 1, BlockType::Indestructible));

        let blocks_buf = Some(my_world.add_blocks_to_fb(&mut builder));

        let state =
            VisibleStateBuf::create(&mut builder, &VisibleStateBufArgs { blocks: blocks_buf });
        builder.finish(state, None);

        let serialized_blocks = get_root_as_visible_state_buf(builder.finished_data());

        assert!(serialized_blocks.blocks().is_some());
        let blocks = serialized_blocks.blocks().unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks.get(0).x(), 0);
        assert_eq!(blocks.get(0).y(), 0);
        assert_eq!(blocks.get(0).block_type(), BlockType::Destructible);
        assert_eq!(blocks.get(1).x(), 0);
        assert_eq!(blocks.get(1).y(), 1);
        assert_eq!(blocks.get(1).block_type(), BlockType::Indestructible);
    }
}
