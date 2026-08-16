use std::sync::Arc;

use log::{debug, error, info, trace};
use thiserror::Error;
use vivid_core::{
    Clock, Component, World,
    glam::{Vec2, Vec3},
};
use vivid_platform::{KeyboardInput, Platform, PlatformError};
use vivid_render::{Camera, RenderError, Renderer};

use crate::{FrameCtx, FrameSchedule, TickCtx, TickSchedule};

/// Position with interpolation history.
#[derive(Clone, Copy, Debug, Default)]
pub struct Position {
    pub prev: Vec3,
    pub curr: Vec3,
}

impl Component for Position {}

impl Position {
    /// Spawn-time constructor: sets BOTH fields, so a new entity doesn't
    /// interpolate in from the origin on its first rendered frame.
    pub const fn at(p: Vec3) -> Self {
        Self { prev: p, curr: p }
    }

    /// Move without interpolating — for spawns, wraps, and teleports.
    pub fn teleport(&mut self, p: Vec3) {
        self.prev = p;
        self.curr = p;
    }
}

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("platform init failed: {0}")]
    Platform(#[from] PlatformError),
    #[error("renderer init failed: {0}")]
    Render(#[from] RenderError),
    #[error("window was never created")]
    NoWindow,
}

/// Fixed number of pumps to wait for winit to hand us a window before giving up.
const WINDOW_WAIT_PUMPS: u32 = 100;

pub struct Engine {
    platform: Platform,
    renderer: Renderer<'static>,
    clock: Clock,
    pub world: World,
    tick_schedule: TickSchedule,
    frame_schedule: FrameSchedule,
    tick_kb_input: KeyboardInput,
    scratch: Vec<Vec3>,
    quit: bool,
}

impl Engine {
    pub fn new(title: &str) -> Result<Self, EngineError> {
        let mut platform = Platform::new(title)?;

        let mut window = None;
        for _ in 0..WINDOW_WAIT_PUMPS {
            if !platform.pump() {
                return Err(EngineError::NoWindow);
            }

            if let Some(w) = platform.window() {
                window = Some(Arc::clone(w));
                break;
            }
        }
        let window = window.ok_or(EngineError::NoWindow)?;

        let size = window.inner_size();
        let renderer = Renderer::new(window, size.width, size.height)?;
        info!("engine ready at {}x{}", size.width, size.height);

        let mut world = World::new();
        world.register::<Position>();

        Ok(Self {
            platform,
            renderer,
            clock: Clock::new(64),
            world,
            tick_schedule: TickSchedule::new(),
            frame_schedule: FrameSchedule::new(),
            tick_kb_input: KeyboardInput::default(),
            scratch: Vec::new(),
            quit: false,
        })
    }

    /// Sets the simulation rate in Hz. Call before the loop starts.
    pub fn set_tick_rate(&mut self, hz: u32) {
        self.clock = Clock::new(hz);
        debug!("tick rate set to {hz} Hz");
    }

    /// Append a simulation system.
    pub fn on_tick<F>(&mut self, name: &'static str, system: F) -> &mut Self
    where
        F: FnMut(&mut TickCtx<'_>) + 'static,
    {
        self.tick_schedule.add(name, system);
        self
    }

    /// Append a presentation system.
    pub fn on_frame<F>(&mut self, name: &'static str, system: F) -> &mut Self
    where
        F: FnMut(&mut FrameCtx<'_>) + 'static,
    {
        self.frame_schedule.add(name, system);
        self
    }

    pub fn kb_input(&self) -> &KeyboardInput {
        self.platform.kb_input()
    }

    pub fn camera(&mut self) -> &mut Camera {
        &mut self.renderer.camera
    }

    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    pub fn set_clear_color(&mut self, r: f64, g: f64, b: f64) {
        self.renderer.clear_color = wgpu_color(r, g, b);
    }

    pub fn request_quit(&mut self) {
        self.quit = true;
    }

    /// Advance one rendered frame. Returns `false` when the engine should stop.
    ///
    /// Pipeline order:
    /// 1. pump OS events, reconcile surface size
    /// 2. Advance the clock, run the tick schedule N times, swapping events after each tick
    /// 3. Run the frame schedule
    /// 4. draw
    pub fn frame(&mut self) -> bool {
        if self.quit || !self.platform.pump() {
            return false;
        }

        if let Some(window) = self.platform.window() {
            let size = window.inner_size();
            self.renderer.ensure_size(size.width, size.height);
        }

        self.tick_kb_input.merge(self.platform.kb_input());

        let ticks = self.clock.advance();
        let tick_delta = self.clock.tick_seconds();
        for _ in 0..ticks {
            let mut ctx = TickCtx {
                world: &mut self.world,
                input: &self.tick_kb_input,
                delta_time: tick_delta,
                quit: false,
            };

            self.tick_schedule.run(&mut ctx);
            let quit = ctx.quit;

            self.world.swap_events();

            if quit {
                self.quit = true;
                break;
            }
        }
        if ticks > 0 {
            self.tick_kb_input.clear_presses();
        }

        trace!("frame: {ticks} ticks, alpha {:.3}", self.clock.alpha());

        let mut ctx = FrameCtx {
            world: &mut self.world,
            input: self.platform.kb_input(),
            camera: &mut self.renderer.camera,
            delta_time: self.clock.frame_seconds(),
            alpha: self.clock.alpha(),
            quit: false,
        };

        self.frame_schedule.run(&mut ctx);
        if ctx.quit {
            self.quit = true;
        }

        self.draw();
        !self.quit
    }

    fn draw(&mut self) {
        let alpha = self.clock.alpha();
        self.scratch.clear();

        let positions = self.world.pool::<Position>();
        self.scratch
            .extend(positions.iter().map(|(_, p)| p.prev.lerp(p.curr, alpha)));

        if let Err(e) = self.renderer.render(&self.scratch) {
            error!("render failed {e}");
            self.quit = true;
        }
    }
}

fn wgpu_color(r: f64, g: f64, b: f64) -> vivid_render::wgpu::Color {
    vivid_render::wgpu::Color { r, g, b, a: 1.0 }
}

impl core::fmt::Debug for Engine {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Engine")
            .field("tick_systems", &self.tick_schedule)
            .field("frame_systems", &self.frame_schedule)
            .field("world", &self.world)
            .finish_non_exhaustive()
    }
}
