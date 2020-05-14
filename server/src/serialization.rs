use flatbuffers::{FlatBufferBuilder, ForwardsUOffset, Vector, WIPOffset};

use schema::math_generated;
use schema::messages_generated;
use schema::world_generated;

use log::error;

use crate::world::{Tank, World};

pub trait Serializable {
    fn serialize(&self, builder: &mut FlatBufferBuilder, config: &Config) -> Vec<u8>;
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
    pub player: u16,
}

impl Config {
    pub fn new(player: u16) -> Config {
        Config { player }
    }
}

impl<'a> Flatbufferable<'a> for Tank {
    type Buffer = world_generated::Tank<'a>;
    #[allow(unused_variables)]
    fn add_to_fb(
        &self,
        builder: &mut FlatBufferBuilder<'a>,
        config: &Config,
    ) -> WIPOffset<world_generated::Tank<'a>> {
        world_generated::Tank::create(
            builder,
            &world_generated::TankArgs {
                pos: Some(&math_generated::Vec2::new(
                    self.pos_ref().x,
                    self.pos_ref().y,
                )),
            },
        )
    }
}

// TODO: 1) see if it's possible to add a test for this. Currently it's hard because you can't
// get_root for an array. Perhaps make a testing schema that just has the [Tank] field.
// 2) See if we can simplify for all Vec<&T> where T is Flatbufferable. Currently difficult because
//    each T has its own associated Buffer type.
impl<'a> Flatbufferable<'a> for Vec<&Tank> {
    type Buffer = Vector<'a, ForwardsUOffset<world_generated::Tank<'a>>>;
    fn add_to_fb(
        &self,
        builder: &mut FlatBufferBuilder<'a>,
        config: &Config,
    ) -> WIPOffset<Vector<'a, ForwardsUOffset<world_generated::Tank<'a>>>> {
        let mut vec = Vec::new();

        for tank in self.iter() {
            vec.push(tank.add_to_fb(builder, config));
        }

        builder.create_vector(vec.as_slice())
    }
}

impl Serializable for World {
    fn serialize(&self, builder: &mut FlatBufferBuilder, config: &Config) -> Vec<u8> {
        builder.reset();

        let (player, other_tanks): (Vec<&Tank>, Vec<&Tank>) = self
            .tanks_ref()
            .iter()
            .partition(|tank| tank.player() == config.player);

        if player.len() != 1 {
            error!("Could not find player in World.");
            return Vec::new();
        }

        let player = player.get(0).unwrap().add_to_fb(builder, config);
        let other_tanks = other_tanks.add_to_fb(builder, config);

        let world = world_generated::WorldState::create(
            builder,
            &world_generated::WorldStateArgs {
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
        builder.finished_data().to_vec()
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

        let recovered_tank = get_root::<world_generated::Tank>(builder.finished_data());

        assert_eq!(recovered_tank.pos().unwrap().x(), tank.pos_ref().x);
        assert_eq!(recovered_tank.pos().unwrap().y(), tank.pos_ref().y);
    }

    #[test]
    fn world_can_be_serialized() {
        let config = Config::new(0);
        let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);

        let mut world = World::new();

        world.add_tank(Tank::new(0, Position { x: 69.0, y: 420.0 }));
        world.add_tank(Tank::new(1, Position { x: 23.0, y: 54.0 }));
        world.add_tank(Tank::new(2, Position { x: 84.0, y: 34.0 }));

        let world_as_bytes = world.serialize(&mut builder, &config);

        let message = get_root::<messages_generated::MessageRoot>(world_as_bytes.as_ref());
        assert_eq!(
            message.message_type(),
            messages_generated::Message::WorldState
        );
        let recovered_world = message.message_as_world_state().unwrap();

        let player = recovered_world.player().unwrap();
        let others = recovered_world.others().unwrap();
        let tanks = world.tanks_ref();

        assert_eq!(player.pos().unwrap().x(), tanks[0].pos_ref().x);
        assert_eq!(player.pos().unwrap().y(), tanks[0].pos_ref().y);

        assert_eq!(others.len(), 2);
        assert_eq!(others.get(0).pos().unwrap().x(), tanks[1].pos_ref().x);
        assert_eq!(others.get(0).pos().unwrap().y(), tanks[1].pos_ref().y);
        assert_eq!(others.get(1).pos().unwrap().x(), tanks[2].pos_ref().x);
        assert_eq!(others.get(1).pos().unwrap().y(), tanks[2].pos_ref().y);
    }
}
