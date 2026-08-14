use std::{sync::Arc, time::Duration};

use log::{debug, error, info, trace};
use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::PhysicalKey,
    platform::pump_events::{EventLoopExtPumpEvents, PumpStatus},
    window::{Window, WindowId},
};

pub use input::{Key, KeyboardInput};
pub use winit;

mod input;

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("event loop creation failed: {0}")]
    EventLoop(#[from] winit::error::EventLoopError),
}

/// Private [`winit`] callback target:
///
/// Accumulates events target [`Platform`] exposes between pumps.
#[derive(Debug, Default)]
struct Handler {
    title: String,
    window: Option<Arc<Window>>,
    kb_input: KeyboardInput,
    close_requested: bool,
    resized: Option<PhysicalSize<u32>>,
}

impl ApplicationHandler for Handler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        if self.window.is_some() {
            return;
        }

        let attrs = Window::default_attributes().with_title(self.title.clone());
        match event_loop.create_window(attrs).map(Arc::new) {
            Ok(window) => {
                info!("Wndow created, inner size {:?}", window.inner_size());
                self.window = Some(window);
            }
            Err(e) => {
                error!("window creation failed: {e}");
                self.close_requested = true;
            }
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                debug!("close requested");
                self.close_requested = true;
            }
            WindowEvent::Resized(size) => {
                trace!("resized to {size:?}");
                self.resized = Some(size);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        repeat,
                        ..
                    },
                ..
            } => {
                if let Some(key) = Key::from_winit(code) {
                    self.kb_input.on_key(key, state, repeat);
                } else {
                    trace!("unmapped key ignored: {code:?}");
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug)]
pub struct Platform {
    event_loop: EventLoop<()>,
    handler: Handler,
}

impl Platform {
    pub fn new(title: &str) -> Result<Self, PlatformError> {
        let event_loop = EventLoop::builder().build()?;
        Ok(Self {
            event_loop,
            handler: Handler {
                title: title.to_owned(),
                ..Handler::default()
            },
        })
    }

    /// Process all pending OS events without blocking. Call once per render
    /// frame, at the top of the loop. Returns `false` when the app should
    /// shut down (window closed or OS exit).
    pub fn pump(&mut self) -> bool {
        self.handler.kb_input.begin_frame();
        self.handler.resized = None;

        let status = self
            .event_loop
            .pump_app_events(Some(Duration::ZERO), &mut self.handler);

        !(matches!(status, PumpStatus::Exit(_)) || self.handler.close_requested)
    }

    pub fn input(&self) -> &KeyboardInput {
        &self.handler.kb_input
    }

    /// `None` until the first pump (winit creates windows only from within
    /// its callbacks, so the window is born inside `pump`, not `new`).
    pub fn window(&self) -> Option<&Arc<Window>> {
        self.handler.window.as_ref()
    }

    /// The resize that arrived during the latest pump, if any. The renderer
    /// will watch this to reconfigure its surface.
    pub fn resized(&self) -> Option<PhysicalSize<u32>> {
        self.handler.resized
    }
}
