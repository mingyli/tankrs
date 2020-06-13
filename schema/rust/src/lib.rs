#![allow(dead_code, unused_imports, clippy::all, clippy::pedantic)]
#[rustfmt::skip]

pub mod action;
pub mod client_message;
pub mod geometry;
pub mod heartbeat;
pub mod server_message;
pub mod tank;
pub mod world;

pub use action::Action;
pub use client_message::ClientMessage;
pub use geometry::Vec2;
pub use heartbeat::Heartbeat;
pub use server_message::ServerMessage;
pub use tank::Tank;
pub use world::World;
