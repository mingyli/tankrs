#![allow(dead_code, unused_imports, clippy::all, clippy::pedantic)]

// TODO(mluogh): rustfmt::skips can be removed on release of rust 1.44.1 and rust-protobuf > 2.14
#[rustfmt::skip]
pub mod action;
#[rustfmt::skip]
pub mod client_message;
#[rustfmt::skip]
pub mod geometry;
#[rustfmt::skip]
pub mod heartbeat;
#[rustfmt::skip]
pub mod server_message;
#[rustfmt::skip]
pub mod tank;
#[rustfmt::skip]
pub mod world;

pub use action::Action;
pub use client_message::ClientMessage;
pub use geometry::Vec2;
pub use heartbeat::Heartbeat;
pub use server_message::ServerMessage;
pub use tank::Tank;
pub use world::World;
