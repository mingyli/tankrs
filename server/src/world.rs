#![allow(dead_code)]
use std::collections::HashMap;
use std::convert::TryFrom;

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

#[derive(Default)]
pub struct World {
    tanks: HashMap<Uuid, Tank>,
    geometrical_world: DefaultGeometricalWorld<f32>,
    mechanical_world: DefaultMechanicalWorld<f32>,

    bodies: DefaultBodySet<f32>,
    colliders: DefaultColliderSet<f32>,
    joint_constraints: DefaultJointConstraintSet<f32>,
    force_generators: DefaultForceGeneratorSet<f32>,
}

pub struct Tank {
    // TODO: discuss what player would look like and replace.
    player_id: Uuid,

    body_handle: DefaultBodyHandle,
    collider_handle: DefaultColliderHandle,
}

#[derive(Debug)]
pub struct PlayerAction {
    pub control: Vec<action::KeyPress>,
}

pub trait Protobufferable {
    type Proto;

    fn serialize(&self, body_set: &DefaultBodySet<f32>) -> Result<Self::Proto>;
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

    pub fn pos(&self, body_set: &DefaultBodySet<f32>) -> Result<Vec2> {
        let vector = self.body(body_set)?.position().translation.vector;

        Ok(vector)
    }

    pub fn new(
        body_set: &mut DefaultBodySet<f32>,
        collider_set: &mut DefaultColliderSet<f32>,
        player_id: Uuid,
        pos: Vec2,
    ) -> Tank {
        let rigid_body = RigidBodyDesc::<f32>::new()
            .mass(1.0)
            .max_linear_velocity(20.0)
            .linear_damping(1.0)
            .translation(pos)
            .build();

        let body_handle = body_set.insert(rigid_body);

        let shape = ShapeHandle::new(Cuboid::new(Vec2::new(0.5, 0.5)));

        let collider = ColliderDesc::new(shape).build(BodyPartHandle(body_handle, 0));
        let collider_handle = collider_set.insert(collider);

        Tank {
            player_id,
            body_handle,
            collider_handle,
        }
    }

    pub fn apply_controls(
        &mut self,
        body_set: &mut DefaultBodySet<f32>,
        controls: &[action::KeyPress],
    ) -> Result<()> {
        for control in controls {
            let force;
            match control {
                action::KeyPress::UP => force = Vec2::new(0.0, -1.0),
                action::KeyPress::DOWN => force = Vec2::new(0.0, 1.0),
                action::KeyPress::LEFT => force = Vec2::new(-1.0, 0.0),
                action::KeyPress::RIGHT => force = Vec2::new(1.0, 0.0),
                action::KeyPress::UNKNOWN => return Err(anyhow!("Unknown control command.")),
            };

            self.apply_local_force(body_set, force)?;
        }

        Ok(())
    }

    fn body<'a>(&self, body_set: &'a DefaultBodySet<f32>) -> Result<&'a RigidBody<f32>> {
        let body = body_set
            .rigid_body(self.body_handle)
            .ok_or(anyhow!("no body found for this tank"))?;
        Ok(body)
    }

    fn body_mut<'a>(
        &self,
        body_set: &'a mut DefaultBodySet<f32>,
    ) -> Result<&'a mut RigidBody<f32>> {
        let body = body_set
            .rigid_body_mut(self.body_handle)
            .ok_or(anyhow!("no body found for this tank"))?;
        Ok(body)
    }

    fn apply_local_force(
        &self,
        body_set: &mut DefaultBodySet<f32>,
        linear_force: Vec2,
    ) -> Result<()> {
        let tank_body = self.body_mut(body_set)?;
        tank_body.apply_local_force(
            0,
            &Force::new(linear_force, 0.0),
            ForceType::AccelerationChange,
            /*auto_wake_up=*/ true,
        );

        Ok(())
    }
}

impl Protobufferable for Tank {
    type Proto = schema::Tank;

    fn serialize(&self, body_set: &DefaultBodySet<f32>) -> Result<schema::Tank> {
        let mut vec_proto = schema::geometry::Vec2::new();
        let mut proto = schema::Tank::new();
        vec_proto.set_x(self.pos(body_set)?.x);
        vec_proto.set_y(self.pos(body_set)?.y);
        proto.set_position(vec_proto);
        Ok(proto)
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

            bodies: DefaultBodySet::new(),
            colliders: DefaultColliderSet::new(),
            joint_constraints: DefaultJointConstraintSet::new(),
            force_generators: DefaultForceGeneratorSet::new(),
        }
    }

    pub fn body_set(&self) -> &DefaultBodySet<f32> {
        &self.bodies
    }

    pub fn register_player(&mut self, player_id: Uuid) {
        let spawn_pos = Vec2::new(
            rand::thread_rng().gen_range(0.0, 10.0),
            rand::thread_rng().gen_range(0.0, 10.0),
        );
        let tank = Tank::new(&mut self.bodies, &mut self.colliders, player_id, spawn_pos);
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
            tank.apply_controls(&mut self.bodies, &action.control)
        } else {
            Err(anyhow!("Tank for player id {} not found.", player_id))
        }
    }

    pub fn tick(&mut self) {
        self.mechanical_world.step(
            &mut self.geometrical_world,
            &mut self.bodies,
            &mut self.colliders,
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

impl TryFrom<&World> for schema::World {
    type Error = anyhow::Error;

    fn try_from(world: &World) -> Result<Self> {
        let mut proto = schema::World::new();
        for tank in world.tanks() {
            proto.mut_tanks().push(tank.serialize(&world.bodies)?);
        }
        Ok(proto)
    }
}
