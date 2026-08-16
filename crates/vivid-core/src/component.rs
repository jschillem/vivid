use std::{
    any::{Any, TypeId, type_name},
    cell::{Ref, RefMut},
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use log::trace;
use rustc_hash::FxHashMap;

use crate::{Entity, Pool};

/// Marker for types that are storable as components.
pub trait Component: 'static {}

/// Dense index assigned to a component type on first use.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ComponentId(u32);

impl ComponentId {
    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Component interning table: [`TypeId`] -> [`ComponentId`].
#[derive(Debug, Default)]
pub(crate) struct Components {
    /// The core intern map.
    ids: FxHashMap<TypeId, ComponentId>,
    /// Type names maintained for diagnostics.
    names: Vec<&'static str>,
}

impl Components {
    pub(crate) fn get<T: Component>(&self) -> Option<ComponentId> {
        self.ids.get(&TypeId::of::<T>()).copied()
    }

    pub(crate) fn get_or_insert<T: Component>(&mut self) -> ComponentId {
        if let Some(id) = self.get::<T>() {
            return id;
        }

        let id =
            ComponentId(u32::try_from(self.names.len()).expect("component type count exceeds u32"));

        self.ids.insert(TypeId::of::<T>(), id);
        self.names.push(type_name::<T>());
        trace!("registered component {} as {id:?}", type_name::<T>());
        id
    }

    pub(crate) fn name(&self, id: ComponentId) -> &'static str {
        self.names.get(id.index()).copied().unwrap_or("<unknown>")
    }

    pub(crate) fn len(&self) -> usize {
        self.names.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

/// Object-safe view of [`Pool<T>`], letting the world hold heterogeneous pools
/// and operate on them without knowing `T`.
pub(crate) trait ErasedPool: Any {
    fn remove_entity(&mut self, e: Entity);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Component> ErasedPool for Pool<T> {
    #[inline]
    fn remove_entity(&mut self, e: Entity) {
        self.remove(e);
    }

    #[inline]
    fn len(&self) -> usize {
        Pool::len(self)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        Pool::is_empty(self)
    }

    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct PoolRef<'a, T: Component> {
    guard: Ref<'a, Box<dyn ErasedPool>>,
    _marker: PhantomData<T>,
}

impl<'a, T: Component> PoolRef<'a, T> {
    pub(crate) fn new(guard: Ref<'a, Box<dyn ErasedPool>>) -> Self {
        Self {
            guard,
            _marker: PhantomData,
        }
    }
}

impl<T: Component> Deref for PoolRef<'_, T> {
    type Target = Pool<T>;

    fn deref(&self) -> &Self::Target {
        self.guard
            .as_any()
            .downcast_ref::<Pool<T>>()
            .expect("pool slot holds the type its ComponentId was interned for")
    }
}

pub struct PoolMut<'a, T> {
    guard: RefMut<'a, Box<dyn ErasedPool>>,
    _marker: PhantomData<T>,
}

impl<'a, T> PoolMut<'a, T> {
    pub(crate) fn new(guard: RefMut<'a, Box<dyn ErasedPool>>) -> Self {
        Self {
            guard,
            _marker: PhantomData,
        }
    }
}

impl<T: Component> Deref for PoolMut<'_, T> {
    type Target = Pool<T>;

    fn deref(&self) -> &Self::Target {
        self.guard
            .as_any()
            .downcast_ref::<Pool<T>>()
            .expect("pool slot holds the type its ComponentId was interned for")
    }
}

impl<T: Component> DerefMut for PoolMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard
            .as_any_mut()
            .downcast_mut::<Pool<T>>()
            .expect("pool slot holds the type its ComponentId was interned for")
    }
}

impl<T: Component> fmt::Debug for PoolRef<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PoolRef")
            .field("component", &type_name::<T>())
            .field("len", &self.guard.len())
            .finish()
    }
}

impl<T: Component> fmt::Debug for PoolMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PoolMut")
            .field("component", &type_name::<T>())
            .field("len", &self.guard.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Health(u32);
    impl Component for Health {}

    struct Mana(u32);
    impl Component for Mana {}

    #[test]
    fn ids_are_stable_and_distinct() {
        let mut components = Components::default();
        let health = components.get_or_insert::<Health>();
        let mana = components.get_or_insert::<Mana>();

        assert_ne!(health, mana);
        assert_eq!(components.get_or_insert::<Health>(), health, "stable");
        assert_eq!(components.len(), 2, "no duplicate registration");
    }

    #[test]
    fn ids_are_dense_from_zero() {
        let mut components = Components::default();
        assert_eq!(components.get_or_insert::<Health>().index(), 0);
        assert_eq!(components.get_or_insert::<Mana>().index(), 1);
    }

    #[test]
    fn get_does_not_register() {
        let mut components = Components::default();
        assert!(components.get::<Health>().is_none());
        assert_eq!(components.len(), 0);

        components.get_or_insert::<Health>();
        assert!(components.get::<Health>().is_some());
    }

    #[test]
    fn names_are_recorded_for_diagnostics() {
        let mut components = Components::default();
        let id = components.get_or_insert::<Health>();
        assert!(
            components.name(id).ends_with("Health"),
            "got {}",
            components.name(id)
        );
    }

    #[test]
    fn erased_pool_removes_without_knowing_t() {
        let mut ents = crate::entity::Entities::new();
        let e = ents.allocate();

        let mut pool: Pool<Health> = Pool::new();
        pool.insert(e, Health(10));

        // Erase, then operate through the trait object.
        let erased: &mut dyn ErasedPool = &mut pool;
        assert_eq!(erased.len(), 1);
        erased.remove_entity(e);
        assert_eq!(erased.len(), 0);
    }
}
