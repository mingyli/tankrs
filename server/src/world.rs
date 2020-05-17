use std::collections::HashMap;

use rand::Rng;
use uuid::Uuid;

use crate::math::Vec2;

pub struct World {
    tanks: HashMap<Uuid, Tank>,
}

pub struct Tank {
    // TODO: discuss what player would look like and replace.
    player: u16,
    pos: Vec2,
}

pub struct UserAction {
    playerId: u16,
    action: Action,
}

pub enum Action {
    KeyUp,
    KeyDown,
    KeyLeft,
    KeyRight,
}

impl Tank {
    pub fn player(&self) -> u16 {
        self.player
    }

    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    pub fn new(player: u16, pos: Vec2) -> Tank {
        Tank { player, pos }
    }

    pub fn apply_movement(&mut self, action: Action) {
        match action {
            Action::KeyUp => self.pos += Vec2::UP,
            Action::KeyDown => self.pos += Vec2::DOWN,
            Action::KeyLeft => self.pos += Vec2::LEFT,
            Action::KeyRight => self.pos += Vec2::RIGHT,
        }
    }
}

impl World {
    pub fn new() -> World {
        World {
            tanks: HashMap::new(),
        }
    }

    // TODO: replace with add_player after discussion on representing player.
    pub fn register_player(&mut self, playerId: u32) {
        let spawn_pos = Vec2::new(
            rand::thread_rng().gen_range(-10.0, 10.0),
            rand::thread_rng().gen_range(-10.0, 10.0),
        );
        let tank = Tank::new(playerId, spawn_pos);
        self.tanks.insert(playerId, tank);
    }

    pub fn tanks(&self) -> i32 {
        //&Vec<Tank> {
        &self.tanks.values();
    }

    //fn consume_user_actions(&self, &mut Vec<Action>) {
    //for action in ac
    //}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tank_can_be_moved() {
        let mut tank = Tank::new(0, Vec2::new(0.0, 0.0));

        tank.apply_movement(Action::KeyRight);
        assert_eq!(tank.pos, Vec2::new(1.0, 0.0));
        tank.apply_movement(Action::KeyLeft);
        assert_eq!(tank.pos, Vec2::new(0.0, 0.0));
        tank.apply_movement(Action::KeyUp);
        assert_eq!(tank.pos, Vec2::new(0.0, 1.0));
        tank.apply_movement(Action::KeyDown);
        assert_eq!(tank.pos, Vec2::new(0.0, 0.0));
    }
}
