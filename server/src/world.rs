#![allow(dead_code)]
use std::collections::HashMap;

use anyhow::{anyhow, Result};
use log::warn;
use rand::Rng;
use schema::action;
use uuid::Uuid;

use crate::math::Vec2;

// TODO(mluogh): replace with config.toml
const TICKS_PER_SECOND: i8 = 10;
//#[allow(clippy::cast_precision_loss)]
const TIME_PER_TICK: f32 = 1.0 / TICKS_PER_SECOND as f32;
const TIME_PER_TICK_SQUARED: f32 = TIME_PER_TICK * TIME_PER_TICK;

pub struct World {
    tanks: HashMap<Uuid, Tank>,
}

pub struct Tank {
    // TODO: discuss what player would look like and replace.
    player_id: Uuid,
    pos: Vec2,
    velocity: Vec2,
    acceleration: Vec2,
}

#[derive(Debug)]
pub struct PlayerAction {
    pub control: Vec<action::KeyPress>,
}

impl PlayerAction {
    pub fn new(control: Vec<action::KeyPress>) -> PlayerAction {
        PlayerAction { control }
    }
}

impl Tank {
    // TODO(mluogh): replace base accel with config.toml
    // Default acceleration per second (no powerups/boost).
    const BASE_ACCELERATION: f32 = 0.1;

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
            velocity: Vec2::new(0.0, 0.0),
            acceleration: Vec2::new(0.0, 0.0),
        }
    }

    pub fn apply_controls(&mut self, controls: &[action::KeyPress]) -> Result<()> {
        for control in controls {
            match control {
                action::KeyPress::UP => self.acceleration += Vec2::UP,
                action::KeyPress::DOWN => self.acceleration += Vec2::DOWN,
                action::KeyPress::LEFT => self.acceleration += Vec2::LEFT,
                action::KeyPress::RIGHT => self.acceleration += Vec2::RIGHT,
                action::KeyPress::UNKNOWN => return Err(anyhow!("Unknown control command.")),
            };
        }

        self.acceleration = self.acceleration * Self::BASE_ACCELERATION;

        Ok(())
    }

    pub fn update_pos(&mut self) -> Vec2 {
        self.pos += self.velocity * TIME_PER_TICK + self.acceleration * (TIME_PER_TICK_SQUARED);
        self.velocity += self.acceleration * TIME_PER_TICK;
        self.acceleration = Vec2::new(0.0, 0.0);
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
            rand::thread_rng().gen_range(0.0, 10.0),
            rand::thread_rng().gen_range(0.0, 10.0),
        );
        let tank = Tank::new(player_id, spawn_pos);
        self.tanks.insert(player_id, tank);
    }

    pub fn unregister_player(&mut self, player_id: Uuid) {
        self.tanks.remove(&player_id);
    }

    pub fn apply_player_actions(&mut self, actions: &HashMap<Uuid, PlayerAction>) {
        for (player_id, user_action) in actions {
            if let Err(msg) = self.apply_player_action(player_id, &user_action) {
                warn!(
                    "Player {} entered an erroneous control: {:?}",
                    player_id, msg
                );
            }
        }
    }

    fn apply_player_action(&mut self, player_id: &Uuid, action: &PlayerAction) -> Result<()> {
        if let Some(tank) = self.tanks.get_mut(player_id) {
            tank.apply_controls(&action.control)
        } else {
            Err(anyhow!("Tank for player id {} not found.", player_id))
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
    fn tank_can_be_moved() -> Result<()> {
        let mut tank = Tank::new(Uuid::new_v4(), Vec2::new(0.0, 0.0));

        tank.apply_controls(&vec![action::KeyPress::RIGHT])?;
        // Equivalent to two ticks of game.
        let first_x = tank.update_pos().x;
        let second_x = tank.update_pos().x;
        assert!(0.0 < first_x && first_x < second_x);

        tank.apply_controls(&vec![action::KeyPress::LEFT])?;
        let third_x = tank.update_pos().x;
        tank.apply_controls(&vec![action::KeyPress::LEFT])?;
        let fourth_x = tank.update_pos().x;
        assert!(third_x > fourth_x);

        tank.apply_controls(&vec![action::KeyPress::LEFT])?;
        let last_x = tank.update_pos().x;

        assert!(last_x < fourth_x);

        tank.apply_controls(&vec![action::KeyPress::UP])?;
        let first_y = tank.update_pos().y;
        let second_y = tank.update_pos().y;
        assert!(0.0 > first_y && first_y > second_y);

        tank.apply_controls(&vec![action::KeyPress::DOWN])?;
        tank.update_pos();
        tank.apply_controls(&vec![action::KeyPress::DOWN])?;
        let third_y = tank.update_pos().y;
        assert!(third_y > second_y);

        Ok(())
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

        let mut player_actions = HashMap::new();
        player_actions.insert(p1_id, PlayerAction::new(vec![action::KeyPress::UP]));
        player_actions.insert(p2_id, PlayerAction::new(vec![action::KeyPress::DOWN]));

        world.apply_player_actions(&player_actions);
        world.tick();

        let p1_tank = world.tank_for_player(p1_id).unwrap();
        assert!(p1_tank.pos.y < p1_original_y);

        let p2_tank = world.tank_for_player(p2_id).unwrap();
        assert!(p2_tank.pos.y > p2_original_y);
    }
}
