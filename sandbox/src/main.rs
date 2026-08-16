use env_logger::Env;
use log::info;
use rand::RngExt;
use vivid::{Component, Engine, Key, Position, glam::Vec3};

#[derive(Debug, Component)]
struct Velocity(Vec3);

const HALF: f32 = 18.0;
const ENTITY_COUNT: usize = 50;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let mut engine = Engine::new("vivid sandbox")?;
    engine.camera().height = HALF * 2.2;

    // Deterministic-ish spawn: swap to `rand::rngs::StdRng::seed_from_u64(..)`
    // if you want two runs to be byte-identical for comparison.
    let mut rng = rand::rng();
    for _ in 0..ENTITY_COUNT {
        let e = engine.world.spawn();
        let pos = Vec3::new(
            rng.random_range(-HALF..HALF),
            rng.random_range(-HALF..HALF),
            rng.random_range(-2.0..2.0), // slight depth spread
        );
        let vel = Vec3::new(
            rng.random_range(-6.0..6.0),
            rng.random_range(-6.0..6.0),
            0.0, // keep z still for now
        );
        engine.world.insert(e, Position::at(pos));
        engine.world.insert(e, Velocity(vel));
    }
    info!("spawned {ENTITY_COUNT} entities");

    engine.on_tick("integrate", |ctx| {
        let mut positions = ctx.world.pool_mut::<Position>();
        let velocities = ctx.world.pool::<Velocity>();
        for (e, vel) in velocities.iter() {
            if let Some(p) = positions.get_mut(e) {
                p.prev = p.curr;
                p.curr += vel.0 * ctx.delta_time;
            }
        }
    });

    // Wrapping is a TELEPORT: prev must follow curr, or the quad interpolates
    // across the whole screen for one frame (a visible streak).
    engine.on_tick("wrap", |ctx| {
        let mut positions = ctx.world.pool_mut::<Position>();
        for (_, p) in positions.iter_mut() {
            let mut wrapped = p.curr;
            if wrapped.x > HALF {
                wrapped.x = -HALF;
            } else if wrapped.x < -HALF {
                wrapped.x = HALF;
            }
            if wrapped.y > HALF {
                wrapped.y = -HALF;
            } else if wrapped.y < -HALF {
                wrapped.y = HALF;
            }
            if wrapped != p.curr {
                p.teleport(wrapped);
            }
        }
    });

    engine.on_tick("quit_on_escape", |ctx| {
        if ctx.input.pressed(Key::Escape) {
            ctx.request_quit();
        }
    });

    // Frame stage: presentation only. A system may own state across runs
    // (FnMut), which is how this accumulates without touching the World.
    let mut elapsed = 0.0f32;
    let mut frames = 0u32;
    engine.on_frame("stats", move |ctx| {
        elapsed += ctx.delta_time;
        frames += 1;
        if elapsed >= 1.0 {
            info!(
                "{frames} fps | {:.2} ms/frame | alpha {:.2}",
                elapsed * 1000.0 / frames as f32,
                ctx.alpha
            );
            elapsed = 0.0;
            frames = 0;
        }
    });

    while engine.frame() {}
    Ok(())
}
