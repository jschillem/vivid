#![allow(dead_code)]

mod entity;
mod pool;
mod time;
mod world;

// Re-exported because `glam` types throughout the `vivid` API.
//
// Downstream crates use `vivid_core::glam` instead of declaring
// their own glam dependency.
pub use glam;

pub use entity::{Entities, Entity};
pub use pool::{Iter as PoolIter, IterMut as PoolIterMut, Pool};
pub use time::Clock;
pub use world::{Position, Velocity, World};
