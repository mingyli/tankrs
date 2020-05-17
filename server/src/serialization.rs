use anyhow::{anyhow, Context, Result};
use flatbuffers::{FlatBufferBuilder, ForwardsUOffset, Vector, WIPOffset};

use schema::math_generated;
use schema::messages_generated;

use crate::world::{Tank, World};

pub type Buffer = Vec<u8>;

pub trait SerializableAsMessage {
    fn serialize(&self, builder: &mut FlatBufferBuilder, config: &Config) -> Result<Buffer>;
}

pub struct Config {
    pub player_id: u16,
}

impl Config {
    pub fn new(player_id: u16) -> Config {
        Config { player_id }
    }
}
trait Flatbufferable<'buf> {
    type Object;
    fn add_to_fb(
        &self,
        builder: &mut FlatBufferBuilder<'buf>,
        config: &Config,
    ) -> WIPOffset<Self::Object>;
}

impl<'buf> Flatbufferable<'buf> for Tank {
    type Object = messages_generated::Tank<'buf>;
    #[allow(unused_variables)]
    fn add_to_fb(
        &self,
        builder: &mut FlatBufferBuilder<'buf>,
        config: &Config,
    ) -> WIPOffset<messages_generated::Tank<'buf>> {
        messages_generated::Tank::create(
            builder,
            &messages_generated::TankArgs {
                pos: Some(&math_generated::Vec2::new(self.pos().x, self.pos().y)),
            },
        )
    }
}

// TODO: 1) see if it's possible to add a test for this. Currently it's hard because you can't
// get_root for an array. Perhaps make a testing schema that just has the [Tank] field.
// 2) See if we can simplify for all Vec<&T> where T is Flatbufferable. Currently difficult because
//    each T has its own associated Buffer type.
impl<'buf> Flatbufferable<'buf> for Vec<&Tank> {
    type Object = Vector<'buf, ForwardsUOffset<messages_generated::Tank<'buf>>>;
    fn add_to_fb(
        &self,
        builder: &mut FlatBufferBuilder<'buf>,
        config: &Config,
    ) -> WIPOffset<Vector<'buf, ForwardsUOffset<messages_generated::Tank<'buf>>>> {
        let mut vec = Vec::new();

        for tank in self.iter() {
            vec.push(tank.add_to_fb(builder, config));
        }

        builder.create_vector(vec.as_slice())
    }
}

impl SerializableAsMessage for World {
    fn serialize(&self, builder: &mut FlatBufferBuilder, config: &Config) -> Result<Buffer> {
        builder.reset();

        let (player, other_tanks): (Vec<&Tank>, Vec<&Tank>) = self
            .tanks()
            .partition(|tank| tank.player() == config.player_id);

        if player.len() != 1 {
            return Err(anyhow!(
                "Config does not specify unique player; can't serialize for schema."
            ));
        }

        let player = player
            .first()
            .context("Player doesn't exist.")?
            .add_to_fb(builder, config);
        let other_tanks = other_tanks.add_to_fb(builder, config);

        let world = messages_generated::WorldState::create(
            builder,
            &messages_generated::WorldStateArgs {
                player: Some(player),
                others: Some(other_tanks),
            },
        );

        let message = messages_generated::MessageRoot::create(
            builder,
            &messages_generated::MessageRootArgs {
                message_type: messages_generated::Message::WorldState,
                message: Some(world.as_union_value()),
            },
        );

        builder.finish(message, None);
        Ok(builder.finished_data().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec2;
    use flatbuffers::get_root;

    #[test]
    fn tank_can_be_flatbuffered() -> Result<()> {
        let config = Config::new(0);
        let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
        let tank = Tank::new(0, Vec2 { x: 69.0, y: 420.0 });
        let tank_buf = tank.add_to_fb(&mut builder, &config);
        builder.finish(tank_buf, None);

        let recovered_tank = get_root::<messages_generated::Tank>(builder.finished_data());

        assert_eq!(recovered_tank.pos().context("f")?.x(), tank.pos().x);
        assert_eq!(recovered_tank.pos().context("f")?.y(), tank.pos().y);
        Ok(())
    }

    #[test]
    fn world_can_be_serialized() -> Result<()> {
        let config = Config::new(0);
        let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);

        let mut world = World::new();

        world.register_player(0);
        world.register_player(1);
        world.register_player(2);

        let world_as_bytes = world.serialize(&mut builder, &config)?;

        let message = get_root::<messages_generated::MessageRoot>(world_as_bytes.as_ref());
        assert_eq!(
            message.message_type(),
            messages_generated::Message::WorldState
        );
        let recovered_world = message.message_as_world_state().context("f")?;

        let player = recovered_world.player().context("f")?;
        let others = recovered_world.others().context("f")?;
        let tanks = world.tanks();

        //assert_eq!(player.pos().context("f")?.x(), tanks[0].pos().x);
        //assert_eq!(player.pos().context("f")?.y(), tanks[0].pos().y);

        assert_eq!(others.len(), 2);
        //assert_eq!(others.get(0).pos().context("f")?.x(), tanks[1].pos().x);
        //assert_eq!(others.get(0).pos().context("f")?.y(), tanks[1].pos().y);
        //assert_eq!(others.get(1).pos().context("f")?.x(), tanks[2].pos().x);
        //assert_eq!(others.get(1).pos().context("f")?.y(), tanks[2].pos().y);
        Ok(())
    }
}
