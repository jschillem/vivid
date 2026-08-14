use std::any::type_name;
use std::cell::RefCell;
use std::fmt;

use log::trace;

use crate::component::{Components, ErasedPool, PoolMut, PoolRef};
use crate::entity::{Entities, Entity};
use crate::pool::Pool;
use crate::{Component, ComponentId};

/// The game state.
///
/// The world currently only works through static registration, requiring
/// every possible component [`Pool`] to be added to this struct.
#[derive(Default)]
pub struct World {
    pub entities: Entities,
    components: Components,
    /// component pools, index by [`ComponentId`].
    pools: Vec<Option<RefCell<Box<dyn ErasedPool>>>>,
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

        for slot in self.pools.iter_mut().flatten() {
            slot.get_mut().remove_entity(e);
        }

        true
    }

    /// Determines if the provided entity is alive.
    pub fn is_alive(&self, e: Entity) -> bool {
        self.entities.is_alive(e)
    }

    pub fn entitites(&self) -> &Entities {
        &self.entities
    }

    pub fn alive_count(&self) -> usize {
        self.entities.alive_count()
    }

    pub fn register<T: Component>(&mut self) -> ComponentId {
        let id = self.components.get_or_insert::<T>();
        if self.pools.len() <= id.index() {
            self.pools.resize_with(id.index() + 1, || None);
        }
        if self.pools[id.index()].is_none() {
            trace!("creating pool for {}", type_name::<T>());
            self.pools[id.index()] = Some(RefCell::new(
                Box::new(Pool::<T>::new()) as Box<dyn ErasedPool>
            ));
        }
        id
    }

    pub fn is_registered<T: Component>(&self) -> bool {
        self.slot::<T>().is_some()
    }

    pub fn registered_component_count(&self) -> usize {
        self.components.len()
    }

    /// Shared borrow of `T`'s pool.
    ///
    /// # Panics
    /// If `T` was never registered, or its pool is already mutably borrowed.
    pub fn pool<T: Component>(&self) -> PoolRef<'_, T> {
        self.try_pool::<T>()
            .unwrap_or_else(|| panic!("{} is not registered", type_name::<T>()))
    }

    /// Exclusive borrow of `T`'s pool.
    ///
    /// # Panics
    /// If `T` was never registered, or its pool is already borrowed.
    pub fn pool_mut<T: Component>(&self) -> PoolMut<'_, T> {
        self.try_pool_mut::<T>()
            .unwrap_or_else(|| panic!("{} is not registered", type_name::<T>()))
    }

    /// Like [`World::pool`] but [`None`] when `T` is unregistred.
    ///
    /// # Panics
    /// Still panics on a conflicting borrow.
    pub fn try_pool<T: Component>(&self) -> Option<PoolRef<'_, T>> {
        let cell = self.slot::<T>()?;
        Some(PoolRef::new(cell.try_borrow().unwrap_or_else(|_| {
            panic!("{} pool already mutably borrowed", type_name::<T>())
        })))
    }

    /// Like [`World::pool_mut`] but [`None`] when `T` is unregistred. For
    /// engine-side optional components ("draw positions **IF** this world uses them")
    /// rather than for game systems, which sehould register up front.
    ///
    /// # Panics
    /// Still panics on a conflicting borrow.
    pub fn try_pool_mut<T: Component>(&self) -> Option<PoolMut<'_, T>> {
        let cell = self.slot::<T>()?;
        Some(PoolMut::new(cell.try_borrow_mut().unwrap_or_else(|_| {
            panic!("{} pool already borrowed", type_name::<T>())
        })))
    }

    /// Attach or replace `T` on `e`, registering `T` on first use.
    pub fn insert<T: Component>(&mut self, e: Entity, value: T) -> Option<T> {
        self.register::<T>();
        self.pool_mut::<T>().insert(e, value)
    }

    /// Detach `T` from `e`, returning it if present.
    pub fn remove<T: Component>(&mut self, e: Entity) -> Option<T> {
        self.try_pool_mut::<T>()?.remove(e)
    }

    pub fn contains<T: Component>(&self, e: Entity) -> bool {
        self.try_pool::<T>().is_some_and(|p| p.contains(e))
    }

    /// Copy `T` out of `e`. `None` if absent, stale, or unregistered.
    ///
    /// Copies rather than borrows: returning `&T` would mean returning the
    /// pool guard too; that is what [`World::pool`] is for.
    pub fn get<T: Component + Copy>(&self, e: Entity) -> Option<T> {
        self.try_pool::<T>()?.get(e).copied()
    }

    fn slot<T: Component>(&self) -> Option<&RefCell<Box<dyn ErasedPool>>> {
        let id = self.components.get::<T>()?;
        self.pools.get(id.index())?.as_ref()
    }
}

impl fmt::Debug for World {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("World")
            .field("alive_entities", &self.entities.alive_count())
            .field("components", &self.components.len())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Health(u32);
    impl Component for Health {}

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Mana(u32);
    impl Component for Mana {}

    #[test]
    fn insert_get_roundtrip() {
        let mut world = World::new();
        let e = world.spawn();

        assert_eq!(world.insert(e, Health(10)), None);
        assert_eq!(world.get::<Health>(e), Some(Health(10)));
        assert!(world.contains::<Health>(e));
    }

    #[test]
    fn insert_registers_lazily() {
        let mut world = World::new();
        assert!(!world.is_registered::<Health>());

        let e = world.spawn();
        world.insert(e, Health(1));

        assert!(world.is_registered::<Health>());
        assert_eq!(world.registered_component_count(), 1);
    }

    #[test]
    fn unregistered_reads_are_graceful() {
        let world = World::new();
        // try_* and the convenience accessors tolerate unregistered types...
        assert!(world.try_pool::<Health>().is_none());
        assert!(!world.is_registered::<Health>());
    }

    #[test]
    #[should_panic(expected = "is not registered")]
    fn unregistered_pool_panics() {
        // ...but the direct accessor is a hard error: a system asking for a
        // pool that does not exist is a setup bug.
        let world = World::new();
        let _ = world.pool::<Health>();
    }

    #[test]
    fn two_different_pools_borrow_simultaneously() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, Health(10));
        world.insert(e, Mana(5));

        let mut health = world.pool_mut::<Health>();
        let mana = world.pool::<Mana>();

        health.get_mut(e).unwrap().0 += mana.get(e).unwrap().0;
        assert_eq!(health.get(e), Some(&Health(15)));
    }

    #[test]
    #[should_panic(expected = "pool already borrowed")]
    fn double_mutable_borrow_panics() {
        let mut world = World::new();
        world.register::<Health>();

        let _first = world.pool_mut::<Health>();
        let _second = world.pool_mut::<Health>();
    }

    #[test]
    fn despawn_clears_every_pool_without_a_list() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, Health(10));
        world.insert(e, Mana(5));

        assert!(world.despawn(e));

        assert!(!world.is_alive(e));
        assert!(world.pool::<Health>().is_empty());
        assert!(world.pool::<Mana>().is_empty());
    }

    #[test]
    fn despawn_stale_is_noop() {
        let mut world = World::new();
        let e = world.spawn();
        assert!(world.despawn(e));
        assert!(!world.despawn(e));
        assert_eq!(world.alive_count(), 0);
    }

    #[test]
    fn despawn_leaves_others_untouched() {
        let mut world = World::new();
        let a = world.spawn();
        let b = world.spawn();
        world.insert(a, Health(1));
        world.insert(b, Health(2));

        world.despawn(a);

        assert_eq!(world.get::<Health>(b), Some(Health(2)));
        assert_eq!(world.pool::<Health>().len(), 1);
    }
}
