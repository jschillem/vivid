//! Sandbox: disposable proving ground. Exempt from crates/ quality standards.

use std::sync::Arc;

use log::info;
use rand::RngExt;
use vivid_core::glam::Vec2;
use vivid_core::{Clock, Position, Velocity, World};
use vivid_platform::{Key, Platform};
use vivid_render::Renderer;

/// First real system. Note the disjoint-borrow destructure: iterating
/// `world.velocities` while calling `world.positions.get_mut` through `world`
/// itself would be rejected — splitting the borrows at the field level is the
/// pattern every system uses.
fn integrate(world: &mut World, dt: f32) {
    let World {
        positions,
        velocities,
        ..
    } = world;

    for (e, vel) in velocities.iter() {
        if let Some(pos) = positions.get_mut(e) {
            pos.prev = pos.curr;
            pos.curr += vel.0 * dt;
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let mut rng: rand::rngs::StdRng = rand::make_rng();

    let mut platform = Platform::new("vivid sandbox")?;
    let mut world = World::new();

    for i in 0u8..5 {
        let e = world.spawn();
        world.positions.insert(e, Position::at(Vec2::ZERO));
        let vx = rng.random_range(-2.0f32..2.0f32);
        let vy = rng.random_range(-1.0f32..1.0f32);
        world.velocities.insert(e, Velocity(Vec2::new(vx, vy)));
    }

    let mut clock = Clock::new(128);
    let mut tick_count: u64 = 0;
    let mut renderer = None;
    let mut quads: Vec<Vec2> = Vec::new();

    while platform.pump() {
        if renderer.is_none()
            && let Some(window) = platform.window()
        {
            let size = window.inner_size();
            renderer = Some(Renderer::new(Arc::clone(window), size.width, size.height)?);
        }

        if let (Some(r), Some(size)) = (renderer.as_mut(), platform.resized()) {
            r.resize(size.width, size.height);
        }

        if platform.input().pressed(Key::Escape) {
            info!("escape pressed, exiting");
            break;
        }

        // SIM stage: fixed ticks. Gameplay, physics, (eventually) Lua update.
        for _ in 0..clock.advance() {
            integrate(&mut world, clock.tick_seconds());
            tick_count += 1;

            // Heartbeat: once per simulated second, prove things move.
            if tick_count.is_multiple_of(64)
                && let Some((e, pos)) = world.positions.iter().next()
            {
                info!("t={}s {e:?} at {:?}", tick_count / 60, pos.curr);
            }
        }

        // FRAME stage: runs once per render frame with the true frame delta.
        // Camera smoothing, cosmetic particles, UI animation live here —
        // nothing that can affect simulation outcome.
        let _frame_dt = clock.frame_seconds();
        let alpha = clock.alpha();
        quads.clear();

        for (_, pos) in &world.positions {
            quads.push(pos.prev.lerp(pos.curr, alpha));
        }

        if let Some(r) = renderer.as_mut()
            && let Err(e) = r.render(&quads)
        {
            log::error!("render failed: {e}");
        }
    }

    info!("shutting down after {tick_count} ticks");
    Ok(())
}
