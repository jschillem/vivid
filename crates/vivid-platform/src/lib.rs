use thiserror::Error;

pub use winit;

mod input;

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("event loop creation failed: {0}")]
    EventLoop(#[from] winit::error::EventLoopError),
}
