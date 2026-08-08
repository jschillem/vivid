use glam::Vec2;

use crate::entity::{Entities, Entity};
use crate::pool::Pool;

/// The game state.
///
/// The world currently only works through static registration, requiring
/// every possible component [`Pool`] to be added to this struct.
#[derive(Debug, Default)]
pub struct World {
    pub entities: Entities,
    pub positions: Pool<Position>,
    pub velocities: Pool<Velocity>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate a bare entity with no components.
    pub fn spawn(&mut self) -> Entity {
        self.entities.allocate()
    }

    /// Remove `e` from **EVERY** pool and deallocate the entity.
    ///
    /// Returns whether `e` was alive.
    pub fn despawn(&mut self, e: Entity) -> bool {
        if !self.entities.deallocate(e) {
            return false;
        }

        // remove all components
        self.positions.remove(e);
        self.velocities.remove(e);

        true
    }

    /// Determines if the provided entity is alive.
    pub fn is_alive(&self, e: Entity) -> bool {
        self.entities.is_alive(e)
    }
}

/// Spatial state.
///
/// Stored as previous + current for the fixed-timestep frame loop:
/// simulation ticks write `curr`, the renderer draws `prev + (curr - prev) * alpha`
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Position {
    pub prev: Vec2,
    pub curr: Vec2,
}

impl Position {
    /// Create a new position point. Initializes both the `prev` and `curr` points.
    pub const fn at(p: Vec2) -> Self {
        Self { prev: p, curr: p }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Velocity(pub Vec2);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_is_alive() {
        let mut world = World::new();
        let e = world.spawn();
        assert!(world.is_alive(e));
        assert_eq!(world.entities.alive_count(), 1);
    }

    #[test]
    fn despawn_clears_all_pools() {
        let mut world = World::new();
        let e = world.spawn();
        world.positions.insert(e, Position::at(Vec2::new(1.0, 2.0)));
        world.velocities.insert(e, Velocity(Vec2::new(0.5, 0.0)));

        assert!(world.despawn(e));

        assert!(!world.is_alive(e));
        assert!(!world.positions.contains(e));
        assert!(!world.velocities.contains(e));
        assert!(world.positions.is_empty());
        assert!(world.velocities.is_empty());
    }

    #[test]
    fn despawn_stale_is_noop() {
        let mut world = World::new();
        let e = world.spawn();
        world.positions.insert(e, Position::default());

        assert!(world.despawn(e));
        assert!(!world.despawn(e), "second despawn must report false");
        assert_eq!(world.entities.alive_count(), 0);
    }

    #[test]
    fn despawn_leaves_other_entities_untouched() {
        let mut world = World::new();
        let a = world.spawn();
        let b = world.spawn();
        world.positions.insert(a, Position::at(Vec2::new(1.0, 0.0)));
        world.positions.insert(b, Position::at(Vec2::new(2.0, 0.0)));

        world.despawn(a);

        assert!(world.is_alive(b));
        assert_eq!(
            world.positions.get(b),
            Some(&Position::at(Vec2::new(2.0, 0.0)))
        );
    }

    #[test]
    fn position_at_sets_prev_and_curr() {
        let p = Position::at(Vec2::new(3.0, 4.0));
        assert_eq!(p.prev, p.curr);
    }
}
