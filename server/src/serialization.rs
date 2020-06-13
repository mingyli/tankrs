use crate::world;

pub trait Protobufferable {
    type Proto;

    fn serialize(&self) -> Self::Proto;
}

impl Protobufferable for world::Tank {
    type Proto = schema::Tank;

    fn serialize(&self) -> schema::Tank {
        let mut vec_proto = schema::geometry::Vec2::new();
        let mut proto = schema::Tank::new();
        vec_proto.set_x(self.pos().x);
        vec_proto.set_y(self.pos().y);
        proto.set_position(vec_proto);
        proto
    }
}

impl Protobufferable for world::World {
    type Proto = schema::World;

    fn serialize(&self) -> schema::World {
        let mut proto = schema::World::new();
        for tank in self.tanks() {
            proto.mut_tanks().push(tank.serialize());
        }
        proto
    }
}
