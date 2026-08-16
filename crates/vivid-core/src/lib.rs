#![allow(dead_code)]

mod component;
mod entity;
mod event;
mod pool;
mod time;
mod world;

// Re-exported because `glam` types are used throughout the `vivid` API.
// Downstream crates use `vivid_core::glam` instead of declaring
// their own glam dependency.
pub use glam;

pub use component::{Component, ComponentId};
pub use entity::{Entities, Entity};
pub use event::{Event, EventId, Events};
pub use pool::{Iter as PoolIter, IterMut as PoolIterMut, Pool};
pub use time::Clock;
pub use world::World;

pub use vivid_derive::{Component, Event};
