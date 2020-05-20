use std::collections::HashMap;

use rand::Rng;
use schema::actions_generated::Movement;
use uuid::Uuid;

use crate::math::Vec2;

// TODO(mluogh): replace with config.toml
const TICKS_PER_SECOND: i32 = 10;

pub struct World {
    tanks: HashMap<Uuid, Tank>,
}

pub struct Tank {
    // TODO: discuss what player would look like and replace.
    player_id: Uuid,
    pos: Vec2,
    vel: Vec2,
}

pub struct PlayerAction {
    pub player_id: Uuid,
    pub action: Movement,
}

impl PlayerAction {
    pub fn new(player_id: Uuid, action: Movement) -> PlayerAction {
        PlayerAction { player_id, action }
    }
}

impl Tank {
    // TODO(mluogh): replace base accel with config.toml
    // Default acceleration per second (no powerups/boost).
    const BASE_ACCELERATION: f32 = 1.0;
    const BASE_ACCEL_PER_TICK: f32 = Self::BASE_ACCELERATION / (TICKS_PER_SECOND as f32);

    pub fn player(&self) -> Uuid {
        self.player_id
    }

    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    pub fn new(player_id: Uuid, pos: Vec2) -> Tank {
        Tank {
            player_id,
            pos,
            vel: Vec2::new(0.0, 0.0),
        }
    }

    pub fn apply_controls(&mut self, movement: Movement) {
        match movement {
            Movement::KeyUp => self.vel += Vec2::UP * Self::BASE_ACCEL_PER_TICK,
            Movement::KeyDown => self.vel += Vec2::DOWN * Self::BASE_ACCEL_PER_TICK,
            Movement::KeyLeft => self.vel += Vec2::LEFT * Self::BASE_ACCEL_PER_TICK,
            Movement::KeyRight => self.vel += Vec2::RIGHT * Self::BASE_ACCEL_PER_TICK,
        }
    }

    pub fn update_pos(&mut self) -> Vec2 {
        self.pos += self.vel;
        self.pos
    }
}

impl World {
    pub fn new() -> World {
        World {
            tanks: HashMap::new(),
        }
    }

    pub fn register_player(&mut self, player_id: Uuid) {
        let spawn_pos = Vec2::new(
            rand::thread_rng().gen_range(-10.0, 10.0),
            rand::thread_rng().gen_range(-10.0, 10.0),
        );
        let tank = Tank::new(player_id, spawn_pos);
        self.tanks.insert(player_id, tank);
    }

    pub fn unregister_player(&mut self, player_id: Uuid) {
        self.tanks.remove(&player_id);
    }

    pub fn apply_player_actions(&mut self, actions: Vec<PlayerAction>) {
        for user_action in actions {
            match self.tanks.get_mut(&user_action.player_id) {
                Some(tank) => tank.apply_controls(user_action.action),
                None => continue,
            }
        }
    }

    pub fn tick(&mut self) {
        // TODO(mluogh): move this to a free function so that lazer Yi can take advantage of WASM
        // and not write his own code
        for tank in self.tanks.values_mut() {
            tank.update_pos();
        }
    }

    pub fn tank_for_player(&self, player_id: Uuid) -> Option<&Tank> {
        self.tanks.get(&player_id)
    }

    pub fn tanks(&self) -> impl Iterator<Item = &'_ Tank> {
        self.tanks.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tank_can_be_moved() {
        let mut tank = Tank::new(Uuid::new_v4(), Vec2::new(0.0, 0.0));

        tank.apply_controls(Movement::KeyRight);
        // Equivalent to two ticks of game.
        tank.update_pos();
        tank.update_pos();
        assert_eq!(tank.pos, Vec2::new(0.2, 0.0));

        tank.apply_controls(Movement::KeyLeft);
        tank.update_pos();
        tank.apply_controls(Movement::KeyLeft);
        tank.update_pos();
        assert_eq!(tank.pos, Vec2::new(0.1, 0.0));
        tank.apply_controls(Movement::KeyRight);

        tank.apply_controls(Movement::KeyUp);
        tank.update_pos();
        tank.update_pos();
        assert_eq!(tank.pos, Vec2::new(0.1, 0.2));

        tank.apply_controls(Movement::KeyDown);
        tank.update_pos();
        assert_eq!(tank.pos, Vec2::new(0.1, 0.2));
    }

    #[test]
    fn world_can_register_players() {
        let mut world = World::new();

        let p1_id = Uuid::new_v4();
        world.register_player(p1_id);
        let p2_id = Uuid::new_v4();
        world.register_player(p2_id);
        let p3_id = Uuid::new_v4();
        world.register_player(p3_id);

        assert_eq!(world.tank_for_player(p1_id).unwrap().player_id, p1_id);
        assert_eq!(world.tank_for_player(p2_id).unwrap().player_id, p2_id);
        assert_eq!(world.tank_for_player(p3_id).unwrap().player_id, p3_id);
    }

    #[test]
    fn world_can_apply_player_movements() {
        let mut world = World::new();

        let p1_id = Uuid::new_v4();
        world.register_player(p1_id);
        let p2_id = Uuid::new_v4();
        world.register_player(p2_id);

        let p1_original_y = world.tank_for_player(p1_id).unwrap().pos.y;
        let p2_original_y = world.tank_for_player(p2_id).unwrap().pos.y;

        let mut player_actions = Vec::new();
        player_actions.push(PlayerAction::new(p1_id, Movement::KeyUp));
        player_actions.push(PlayerAction::new(p2_id, Movement::KeyDown));
        world.apply_player_actions(player_actions);
        world.tick();

        let p1_tank = world.tank_for_player(p1_id).unwrap();
        assert_eq!(p1_tank.pos.y, p1_original_y + 0.1);

        let p2_tank = world.tank_for_player(p2_id).unwrap();
        assert_eq!(p2_tank.pos.y, p2_original_y - 0.1);
    }
}
