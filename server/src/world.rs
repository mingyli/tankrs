#![allow(dead_code)]
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

use anyhow::{anyhow, Result};
use log::warn;
use ncollide2d::shape::{Cuboid, ShapeHandle};
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::math::{Force, ForceType};
use nphysics2d::object::{
    Body, BodyPartHandle, ColliderDesc, DefaultBodyHandle, DefaultBodySet, DefaultColliderHandle,
    DefaultColliderSet, RigidBody, RigidBodyDesc,
};
use nphysics2d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};
use rand::Rng;
use schema::action;
use uuid::Uuid;

type Vec2 = nalgebra::Vector2<f32>;

// TODO(mluogh): replace with config.toml
const TICKS_PER_SECOND: i8 = 10;
//#[allow(clippy::cast_precision_loss)]
const TIME_PER_TICK: f32 = 1.0 / TICKS_PER_SECOND as f32;
const TIME_PER_TICK_SQUARED: f32 = TIME_PER_TICK * TIME_PER_TICK;

pub struct World {
    tanks: HashMap<Uuid, Tank>,
    geometrical_world: DefaultGeometricalWorld<f32>,
    mechanical_world: DefaultMechanicalWorld<f32>,

    bodies: Rc<RefCell<DefaultBodySet<f32>>>,
    colliders: Rc<RefCell<DefaultColliderSet<f32>>>,
    joint_constraints: DefaultJointConstraintSet<f32>,
    force_generators: DefaultForceGeneratorSet<f32>,
}

pub struct Tank {
    // TODO: discuss what player would look like and replace.
    player_id: Uuid,

    body_handle: DefaultBodyHandle,
    collider_handle: DefaultColliderHandle,

    body_set: Rc<RefCell<DefaultBodySet<f32>>>,
    collider_set: Rc<RefCell<DefaultColliderSet<f32>>>,
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
    // TODO(mluogh): replace this constant with a config.toml

    pub fn player(&self) -> Uuid {
        self.player_id
    }

    pub fn pos(&self) -> Result<Vec2> {
        let vector = self.get_body()?.position().translation.vector;

        Ok(vector)
    }

    pub fn new(
        body_set: Rc<RefCell<DefaultBodySet<f32>>>,
        collider_set: Rc<RefCell<DefaultColliderSet<f32>>>,
        player_id: Uuid,
        pos: Vec2,
    ) -> Tank {
        let rigid_body = RigidBodyDesc::<f32>::new()
            .mass(1.0)
            .max_linear_velocity(20.0)
            .linear_damping(1.0)
            .translation(pos)
            .build();

        let body_handle = body_set.borrow_mut().insert(rigid_body);

        let shape = ShapeHandle::new(Cuboid::new(Vec2::new(0.5, 0.5)));

        let collider = ColliderDesc::new(shape).build(BodyPartHandle(body_handle, 0));
        let collider_handle = collider_set.borrow_mut().insert(collider);

        Tank {
            player_id,
            body_handle,
            collider_handle,
            body_set,
            collider_set,
        }
    }

    pub fn apply_controls(&mut self, controls: &[action::KeyPress]) -> Result<()> {
        for control in controls {
            match control {
                action::KeyPress::UP => self.apply_local_force(Vec2::new(0.0, -1.0))?,
                action::KeyPress::DOWN => self.apply_local_force(Vec2::new(0.0, 1.0))?,
                action::KeyPress::LEFT => self.apply_local_force(Vec2::new(-1.0, 0.0))?,
                action::KeyPress::RIGHT => self.apply_local_force(Vec2::new(1.0, 0.0))?,
                action::KeyPress::UNKNOWN => return Err(anyhow!("Unknown control command.")),
            };
        }

        Ok(())
    }

    fn get_body(&self) -> Result<Ref<RigidBody<f32>>> {
        if let None = self.body_set.borrow().rigid_body(self.body_handle) {
            return Err(anyhow!("no body found for this tank"));
        }

        let body = Ref::map(self.body_set.borrow(), |body_set| {
            body_set.rigid_body(self.body_handle).unwrap()
        });

        Ok(body)
    }

    fn get_body_mut(&self) -> Result<RefMut<RigidBody<f32>>> {
        if let None = self.body_set.borrow().rigid_body(self.body_handle) {
            return Err(anyhow!("no body found for this tank"));
        }

        let body = RefMut::map(self.body_set.borrow_mut(), |body_set| {
            body_set.rigid_body_mut(self.body_handle).unwrap()
        });

        Ok(body)
    }

    fn apply_local_force(&self, linear_force: Vec2) -> Result<()> {
        let mut tank_body = self.get_body_mut()?;
        // tank_body.
        // warn!("{:?}", tank_body.
        tank_body.apply_local_force(
            0,
            &Force::new(linear_force, 0.0),
            ForceType::AccelerationChange,
            /*auto_wake_up=*/ true,
        );

        Ok(())
    }
}

impl World {
    pub fn new() -> World {
        let mut mech_world = DefaultMechanicalWorld::new(Vec2::new(0.0, 0.0));
        mech_world.set_timestep(TIME_PER_TICK);
        World {
            tanks: HashMap::new(),
            geometrical_world: DefaultGeometricalWorld::new(),
            mechanical_world: mech_world,

            bodies: Rc::new(RefCell::new(DefaultBodySet::new())),
            colliders: Rc::new(RefCell::new(DefaultColliderSet::new())),
            joint_constraints: DefaultJointConstraintSet::new(),
            force_generators: DefaultForceGeneratorSet::new(),
        }
    }

    pub fn register_player(&mut self, player_id: Uuid) {
        let spawn_pos = Vec2::new(
            rand::thread_rng().gen_range(0.0, 10.0),
            rand::thread_rng().gen_range(0.0, 10.0),
        );
        let tank = Tank::new(
            Rc::clone(&self.bodies),
            Rc::clone(&self.colliders),
            player_id,
            spawn_pos,
        );
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
        self.mechanical_world.step(
            &mut self.geometrical_world,
            &mut *self.bodies.borrow_mut(),
            &mut *self.colliders.borrow_mut(),
            &mut self.joint_constraints,
            &mut self.force_generators,
        );
    }

    pub fn tank_for_player(&self, player_id: Uuid) -> Option<&Tank> {
        self.tanks.get(&player_id)
    }

    pub fn tanks(&self) -> impl Iterator<Item = &'_ Tank> {
        self.tanks.values()
    }
}
