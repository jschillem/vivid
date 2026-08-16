mod engine;
mod schedule;

pub use engine::{Engine, EngineError, Position};
pub use schedule::{FrameCtx, FrameSchedule, Schedule, TickCtx, TickSchedule};

pub use vivid_core::{self as core, Clock, Component, Entity, Event, Pool, World, glam};
pub use vivid_platform::{self as platform, Key, KeyboardInput};
pub use vivid_render::{self as render, Camera, Renderer};

pub mod prelude {
    pub use crate::{Engine, FrameCtx, Position, TickCtx};
    pub use vivid_core::glam::{Vec2, Vec3};
    pub use vivid_core::{Clock, Component, Entity, Event, World};
    pub use vivid_platform::Key;
}
