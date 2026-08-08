use core::mem;
use std::{
    iter::{Copied, FusedIterator, Zip},
    slice,
};

use log::{debug, trace, warn};

use crate::entity::Entity;

/// Sentinel in `sparse` meaning "no entry".
///
/// There is not a memory niche for a u32, so using Option<u32>
/// would double the sparse array size.
const NO_SLOT: u32 = u32::MAX;

/// Sparse-set storage for one component type.
///
/// INVARIANTS:
/// - `dense` and `dense_entities` always have the same length, with no holes.
/// - For every live entry at dense slot `s`: `sparse[dense_entities[s].index()] == s`.
/// - `sparse` may contain stale garbage: validity is always confirmed by comparing the
///   **FULL** [`Entity`] at `dense_entities[slot]` against the query.
/// - Dense order is **UNSTABLE**. Dense slots must never be stored anywhere outside of this struct.
/// - At most one live entry per entity **INDEX**. If an insert finds a lingering entry from a
///   previous generation at the same index (a component that was never removed before its entity
///   died), the stale entry is evicted first. see [`Pool::insert`].
#[derive(Debug)]
pub struct Pool<T> {
    sparse: Vec<u32>,
    dense_entities: Vec<Entity>,
    dense: Vec<T>,
}

impl<T> Pool<T> {
    /// Create a new sparse-set component pool.
    pub const fn new() -> Self {
        Self {
            sparse: Vec::new(),
            dense_entities: Vec::new(),
            dense: Vec::new(),
        }
    }

    /// The live component count.
    pub fn len(&self) -> usize {
        self.dense.len()
    }

    /// If there is no active components.
    pub fn is_empty(&self) -> bool {
        self.dense.is_empty()
    }

    /// Determines if an entity contains this component.
    pub fn contains(&self, e: Entity) -> bool {
        self.slot_of(e).is_some()
    }

    /// Insert a new component.
    ///
    /// If `e` already has this component, replaces it and returns the old value.
    pub fn insert(&mut self, e: Entity, value: T) -> Option<T> {
        if let Some(slot) = self.slot_of(e) {
            debug!("insert replaced existing component for {e:?}");
            return Some(mem::replace(&mut self.dense[slot], value));
        }

        let idx = e.index() as usize;
        if idx >= self.sparse.len() {
            self.sparse.resize(idx + 1, NO_SLOT);
        } else if self.sparse[idx] != NO_SLOT {
            // a previous generation at this index died without its component
            // being removed (entity deallocated without World::despawn?).
            // Left alone it would become unreachable via get/remove but still
            // visible to iteration, so we must evict it now.
            let orphan_slot = self.sparse[idx] as usize;
            let (orphan, _) = self.remove_slot(orphan_slot);
            warn!("evicted orphaned component of dead {orphan:?} on insert for {e:?}");
        }

        let slot = u32::try_from(self.dense.len()).expect("component count exceeds u32::MAX");
        self.sparse[idx] = slot;
        self.dense_entities.push(e);
        self.dense.push(value);
        trace!("insert component for {e:?} at slot {slot}");
        None
    }

    /// Gets the entity's component if it exists.
    pub fn get(&self, e: Entity) -> Option<&T> {
        self.slot_of(e).map(|s| &self.dense[s])
    }

    /// Gets the entity's component mutably if it exists.
    pub fn get_mut(&mut self, e: Entity) -> Option<&mut T> {
        let slot = self.slot_of(e)?;
        Some(&mut self.dense[slot])
    }

    /// Swap-removes the given entity's component.
    ///
    /// Returns [`None`] if the entity's component is absent or stale.
    pub fn remove(&mut self, e: Entity) -> Option<T> {
        let slot = self.slot_of(e)?;
        let (_, value) = self.remove_slot(slot);
        trace!("remove component for {e:?}");
        Some(value)
    }

    /// Walk of the dense array, associating the entity with its component.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            inner: self.dense_entities.iter().copied().zip(self.dense.iter()),
        }
    }

    /// Walk of the dense array, associating the entity with a mutable reference to its component.
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            inner: self
                .dense_entities
                .iter()
                .copied()
                .zip(self.dense.iter_mut()),
        }
    }

    pub fn slot_of(&self, e: Entity) -> Option<usize> {
        let slot = *self.sparse.get(e.index() as usize)?;
        if slot == NO_SLOT {
            return None;
        }

        let slot = slot as usize;
        (self.dense_entities.get(slot).copied() == Some(e)).then_some(slot)
    }

    /// Core swap-remove at a known-valid dense slot. Returns the evicted
    /// (owner, value) pair.
    fn remove_slot(&mut self, slot: usize) -> (Entity, T) {
        let value = self.dense.swap_remove(slot);
        let owner = self.dense_entities.swap_remove(slot);

        if slot < self.dense.len() {
            // Someone from the tail was swapped into `slot`; we need to repoint them.
            let moved = self.dense_entities[slot];
            self.sparse[moved.index() as usize] =
                u32::try_from(slot).expect("dense slots fit in u32");
        }
        self.sparse[owner.index() as usize] = NO_SLOT;
        (owner, value)
    }
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over `(Entity, &T)`.
#[derive(Debug)]
pub struct Iter<'a, T> {
    inner: Zip<Copied<slice::Iter<'a, Entity>>, slice::Iter<'a, T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (Entity, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth(n)
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.fold(init, f)
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {}
impl<T> FusedIterator for Iter<'_, T> {}

impl<T> DoubleEndedIterator for Iter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<'a, T> IntoIterator for &'a Pool<T> {
    type Item = (Entity, &'a T);
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator over `(Entity, &mut T)`.
#[derive(Debug)]
pub struct IterMut<'a, T> {
    inner: Zip<Copied<slice::Iter<'a, Entity>>, slice::IterMut<'a, T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (Entity, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth(n)
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.fold(init, f)
    }
}

impl<T> ExactSizeIterator for IterMut<'_, T> {}
impl<T> FusedIterator for IterMut<'_, T> {}

impl<T> DoubleEndedIterator for IterMut<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<'a, T> IntoIterator for &'a mut Pool<T> {
    type Item = (Entity, &'a mut T);
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::Entities;

    fn three(ents: &mut Entities) -> (Entity, Entity, Entity) {
        (ents.allocate(), ents.allocate(), ents.allocate())
    }

    #[test]
    fn insert_get_roundtrip() {
        let mut ents = Entities::new();

        let e = ents.allocate();
        let mut pool: Pool<i32> = Pool::new();

        assert_eq!(pool.insert(e, 7), None);
        assert_eq!(pool.get(e), Some(&7));
        assert!(pool.contains(e));
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn insert_existing_replaces_and_returns_old() {
        let mut ents = Entities::new();
        let e = ents.allocate();
        let mut pool: Pool<i32> = Pool::new();

        pool.insert(e, 1);
        assert_eq!(pool.insert(e, 2), Some(1));
        assert_eq!(pool.get(e), Some(&2));
        assert_eq!(pool.len(), 1, "replace must not grow the pool");
    }

    #[test]
    fn remove_middle_fixes_up_swapped_tail() {
        let mut ents = Entities::new();
        let (a, b, c) = three(&mut ents);
        let mut pool: Pool<i32> = Pool::new();
        pool.insert(a, 1);
        pool.insert(b, 2);
        pool.insert(c, 3);

        assert_eq!(pool.remove(a), Some(1));

        // White-box: c (the tail) must now occupy a's old slot 0, and its
        // sparse entry must have followed it.
        assert_eq!(pool.sparse[c.index() as usize], 0);
        assert_eq!(pool.sparse[a.index() as usize], NO_SLOT);

        assert_eq!(pool.get(b), Some(&2));
        assert_eq!(pool.get(c), Some(&3));
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn remove_last_element() {
        let mut ents = Entities::new();
        let (a, b, _) = three(&mut ents);
        let mut pool: Pool<i32> = Pool::new();

        // Tail removal with survivors: the no-swap branch.
        pool.insert(a, 1);
        pool.insert(b, 2);
        assert_eq!(pool.remove(b), Some(2));
        assert_eq!(pool.get(a), Some(&1));
        assert_eq!(pool.sparse[b.index() as usize], NO_SLOT);

        // Sole-element removal.
        assert_eq!(pool.remove(a), Some(1));
        assert!(pool.is_empty());
    }

    #[test]
    fn remove_absent_is_none() {
        let mut ents = Entities::new();
        let e = ents.allocate();
        let mut pool: Pool<i32> = Pool::new();

        assert_eq!(pool.remove(e), None);
    }

    #[test]
    fn remove_then_reinsert() {
        let mut ents = Entities::new();
        let e = ents.allocate();
        let mut pool: Pool<i32> = Pool::new();

        pool.insert(e, 1);
        pool.remove(e);
        assert_eq!(pool.insert(e, 2), None, "reinsert is a fresh insert");
        assert_eq!(pool.get(e), Some(&2));
    }

    #[test]
    fn stale_generation_get_is_none() {
        let mut ents = Entities::new();
        let stale = ents.allocate();
        let mut pool: Pool<i32> = Pool::new();
        pool.insert(stale, 1);

        pool.remove(stale);
        ents.deallocate(stale);
        let fresh = ents.allocate();
        assert_eq!(stale.index(), fresh.index(), "test requires index reuse");
        pool.insert(fresh, 2);

        assert_eq!(pool.get(stale), None, "same index, old generation");
        assert!(!pool.contains(stale));
        assert_eq!(pool.get(fresh), Some(&2));
    }

    #[test]
    fn stale_insert_evicts_orphan() {
        let mut ents = Entities::new();
        let old = ents.allocate();
        let mut pool: Pool<i32> = Pool::new();
        pool.insert(old, 1);

        // Entity dies WITHOUT its component being removed (despawn skipped).
        ents.deallocate(old);
        let new = ents.allocate();
        assert_eq!(old.index(), new.index(), "test requires index reuse");

        pool.insert(new, 2);
        assert_eq!(pool.len(), 1, "orphan must be evicted, not leaked");
        assert_eq!(pool.get(new), Some(&2));
        let live: Vec<Entity> = pool.iter().map(|(e, _)| e).collect();
        assert_eq!(live, vec![new], "iteration must not see the orphan");
    }

    #[test]
    fn iter_visits_exactly_live_set() {
        let mut ents = Entities::new();
        let (a, b, c) = three(&mut ents);
        let mut pool: Pool<i32> = Pool::new();
        pool.insert(a, 1);
        pool.insert(b, 2);
        pool.insert(c, 3);
        pool.remove(b);

        let mut seen: Vec<(Entity, i32)> = pool.iter().map(|(e, v)| (e, *v)).collect();
        seen.sort_by_key(|(e, _)| e.index());
        assert_eq!(seen, vec![(a, 1), (c, 3)]);
    }

    #[test]
    fn iter_mut_mutates_in_place() {
        let mut ents = Entities::new();
        let (a, b, _) = three(&mut ents);
        let mut pool: Pool<i32> = Pool::new();
        pool.insert(a, 1);
        pool.insert(b, 2);

        for (_, v) in &mut pool {
            *v += 10;
        }
        assert_eq!(pool.get(a), Some(&11));
        assert_eq!(pool.get(b), Some(&12));
    }
}
