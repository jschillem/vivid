use std::num::NonZeroU32;

use log::{debug, trace, warn};

const GENERATION_ONE: NonZeroU32 = NonZeroU32::MIN;

/// Opaque id for an entity.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Entity {
    index: u32,
    generation: NonZeroU32,
}

impl Entity {
    /// The entity's slot index.
    #[inline]
    #[must_use]
    pub const fn index(self) -> u32 {
        self.index
    }

    /// The entity's generation count.
    #[inline]
    #[must_use]
    pub const fn generation(self) -> NonZeroU32 {
        self.generation
    }

    /// Pack the entity id into a single 64 bit value.
    #[inline]
    #[must_use]
    pub fn to_bits(self) -> u64 {
        (u64::from(self.generation.get()) << 32) | u64::from(self.index)
    }

    /// Unpack from arbitrary bits.
    ///
    /// Returns [`None`] if the generation half is zero.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn from_bits(bits: u64) -> Option<Self> {
        let index = (bits & u64::from(u32::MAX)) as u32;
        let Some(generation) = NonZeroU32::new((bits >> 32) as u32) else {
            debug!("rejected entity bits {bits:#018x}: zero generation");
            return None;
        };

        Some(Self { index, generation })
    }
}

/// Entity allocator.
///
/// INVARIANT: `generations[i]` holds the generation of the currently-alive
/// entity at index `i`, OR (if `i` is dead / on the free list) the generation
/// that the NEXT entity born at `i` will receive. Generations are bumpled at
/// DEATH, in [`Entities::deallocate`]. Therefore [`Entities::is_alive`] needs
/// only the generation check and never consults `free`.
///
/// `free` is LIFO and purely an allocation-reuse structure.
///
#[derive(Debug, Default)]
pub struct Entities {
    generations: Vec<NonZeroU32>, // indexed by entity index
    free: Vec<u32>,               // dead indices awaiting reuse, LIFO
}

impl Entities {
    /// Create a new [`Entity`] allocator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate a new entity.
    ///
    /// This allocation is O(1) amortized. Reuses a dead index if one exists
    /// (its generation was pre-bumped at death); otherwise appends a fresh
    /// index at generation 1.
    #[must_use = "An untracked entity will never be deallocated"]
    pub fn allocate(&mut self) -> Entity {
        let e = if let Some(index) = self.free.pop() {
            Entity {
                index,
                generation: self.generations[index as usize],
            }
        } else {
            let index = u32::try_from(self.generations.len())
                .expect("entity index space exhausted (> u32::MAX slots)");

            self.generations.push(GENERATION_ONE);
            Entity {
                index,
                generation: GENERATION_ONE,
            }
        };

        trace!("allocated {e:?}");
        e
    }

    /// Deallocates a given [`Entity`].
    ///
    /// Returns `true` iff `e` was alive and is now dead.
    /// Stale, dead, or never-issued will return `false.`
    pub fn deallocate(&mut self, e: Entity) -> bool {
        if !self.is_alive(e) {
            debug!("deallocate of dead/stale {e:?} ignored");
            return false;
        }

        let slot = &mut self.generations[e.index as usize];
        *slot = slot.checked_add(1).unwrap_or_else(|| {
            warn!(
                "generation wrapped for index {}; ancient stale handles may alias",
                e.index
            );
            GENERATION_ONE
        });

        self.free.push(e.index);
        trace!("deallocated {e:?}");
        true
    }

    /// Determines if the given [`Entity`] is alive.
    pub fn is_alive(&self, e: Entity) -> bool {
        self.generations
            .get(e.index as usize)
            .is_some_and(|&g| g == e.generation)
    }

    /// Number of currently-alive entities.
    pub fn alive_count(&self) -> usize {
        self.generations.len() - self.free.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reuse_bumps_generation() {
        let mut ents = Entities::new();
        let a = ents.allocate();
        assert!(ents.deallocate(a));

        let b = ents.allocate();
        assert_eq!(a.index(), b.index(), "index should be recycled (LIFO)");
        assert_ne!(a.generation(), b.generation());
    }

    #[test]
    fn stale_handle_dead_new_handle_alive() {
        let mut ents = Entities::new();
        let a = ents.allocate();
        ents.deallocate(a);

        let b = ents.allocate();

        assert!(!ents.is_alive(a));
        assert!(ents.is_alive(b));
    }

    #[test]
    fn double_deallocate_is_noop_and_returns_false() {
        let mut ents = Entities::new();
        let a = ents.allocate();
        assert!(ents.deallocate(a));

        let gen_after_first = ents.generations[a.index() as usize];
        assert!(!ents.deallocate(a));
        assert_eq!(
            ents.generations[a.index() as usize],
            gen_after_first,
            "Second deallocate must not bump again"
        );

        assert_eq!(ents.free.len(), 1, "index must not be freed twice");
    }

    #[test]
    fn is_alive_never_issued_index_no_panic() {
        let ents = Entities::new();
        let ghost = Entity {
            index: 999,
            generation: GENERATION_ONE,
        };

        assert!(!ents.is_alive(ghost));
    }

    #[test]
    fn bits_roundtrip() {
        let mut ents = Entities::new();
        let a = ents.allocate();
        ents.deallocate(a);

        let b = ents.allocate();

        assert_eq!(Entity::from_bits(b.to_bits()), Some(b));
    }

    #[test]
    fn from_bits_zero_generation_is_none() {
        assert_eq!(Entity::from_bits(0), None);
        assert_eq!(Entity::from_bits(42), None);
    }

    #[test]
    fn alive_count_tracks_alloc_and_dealloc() {
        let mut ents = Entities::new();

        let a = ents.allocate();
        let _b = ents.allocate();
        assert_eq!(ents.alive_count(), 2);

        ents.deallocate(a);
        assert_eq!(ents.alive_count(), 1);

        let _ = ents.allocate();
        assert_eq!(ents.alive_count(), 2);
    }
}
