use anyhow::{anyhow, Result};
use flatbuffers::{FlatBufferBuilder, ForwardsUOffset, Vector, WIPOffset};

use schema::math_generated;
use schema::messages_generated;

use crate::world::{Tank, World};

pub trait SerializableAsMessage {
    fn serialize(&self, builder: &mut FlatBufferBuilder, config: &Config) -> Result<Vec<u8>>;
}

trait Flatbufferable<'a> {
    type Buffer;
    fn add_to_fb(
        &self,
        builder: &mut FlatBufferBuilder<'a>,
        config: &Config,
    ) -> WIPOffset<Self::Buffer>;
}

pub struct Config {
    pub player_id: u16,
}

impl Config {
    pub fn new(player_id: u16) -> Config {
        Config { player_id }
    }
}

impl<'a> Flatbufferable<'a> for Tank {
    type Buffer = messages_generated::Tank<'a>;
    #[allow(unused_variables)]
    fn add_to_fb(
        &self,
        builder: &mut FlatBufferBuilder<'a>,
        config: &Config,
    ) -> WIPOffset<messages_generated::Tank<'a>> {
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
impl<'a> Flatbufferable<'a> for Vec<&Tank> {
    type Buffer = Vector<'a, ForwardsUOffset<messages_generated::Tank<'a>>>;
    fn add_to_fb(
        &self,
        builder: &mut FlatBufferBuilder<'a>,
        config: &Config,
    ) -> WIPOffset<Vector<'a, ForwardsUOffset<messages_generated::Tank<'a>>>> {
        let mut vec = Vec::new();

        for tank in self.iter() {
            vec.push(tank.add_to_fb(builder, config));
        }

        builder.create_vector(vec.as_slice())
    }
}

impl SerializableAsMessage for World {
    fn serialize(&self, builder: &mut FlatBufferBuilder, config: &Config) -> Result<Vec<u8>> {
        builder.reset();

        let (player, other_tanks): (Vec<&Tank>, Vec<&Tank>) = self
            .tanks()
            .iter()
            .partition(|tank| tank.player() == config.player_id);

        if player.len() != 1 {
            return Err(anyhow!(
                "Config does not specify unique player; can't serialize for schema."
            ));
        }

        let player = player
            .first()
            .ok_or_else(|| anyhow!("Player doesn't exist."))?
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
    use crate::math::Position;
    use flatbuffers::get_root;

    #[test]
    fn tank_can_be_flatbuffered() {
        let config = Config::new(0);
        let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
        let tank = Tank::new(0, Position { x: 69.0, y: 420.0 });
        let tank_buf = tank.add_to_fb(&mut builder, &config);
        builder.finish(tank_buf, None);

        let recovered_tank = get_root::<messages_generated::Tank>(builder.finished_data());

        assert_eq!(recovered_tank.pos().unwrap().x(), tank.pos().x);
        assert_eq!(recovered_tank.pos().unwrap().y(), tank.pos().y);
    }

    #[test]
    fn world_can_be_serialized() {
        let config = Config::new(0);
        let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);

        let mut world = World::new();

        world.add_tank(Tank::new(0, Position { x: 69.0, y: 420.0 }));
        world.add_tank(Tank::new(1, Position { x: 23.0, y: 54.0 }));
        world.add_tank(Tank::new(2, Position { x: 84.0, y: 34.0 }));

        let world_as_bytes = world.serialize(&mut builder, &config).unwrap();

        let message = get_root::<messages_generated::MessageRoot>(world_as_bytes.as_ref());
        assert_eq!(
            message.message_type(),
            messages_generated::Message::WorldState
        );
        let recovered_world = message.message_as_world_state().unwrap();

        let player = recovered_world.player().unwrap();
        let others = recovered_world.others().unwrap();
        let tanks = world.tanks();

        assert_eq!(player.pos().unwrap().x(), tanks[0].pos().x);
        assert_eq!(player.pos().unwrap().y(), tanks[0].pos().y);

        assert_eq!(others.len(), 2);
        assert_eq!(others.get(0).pos().unwrap().x(), tanks[1].pos().x);
        assert_eq!(others.get(0).pos().unwrap().y(), tanks[1].pos().y);
        assert_eq!(others.get(1).pos().unwrap().x(), tanks[2].pos().x);
        assert_eq!(others.get(1).pos().unwrap().y(), tanks[2].pos().y);
    }
}
