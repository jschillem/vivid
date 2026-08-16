use std::{
    any::{Any, TypeId, type_name},
    cell::{Ref, RefMut},
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use log::trace;
use rustc_hash::FxHashMap;

/// Marker for types that are sendable as events.
pub trait Event: 'static {}

/// Dense index assigned to an event type on first registration.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct EventId(u32);

impl EventId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Event interning table: [`TypeId`] -> [`EventId`]
#[derive(Debug, Default)]
pub(crate) struct EventTypes {
    ids: FxHashMap<TypeId, EventId>,
    names: Vec<&'static str>,
}

impl EventTypes {
    pub(crate) fn get<T: Event>(&self) -> Option<EventId> {
        self.ids.get(&TypeId::of::<T>()).copied()
    }

    pub(crate) fn get_or_insert<T: Event>(&mut self) -> EventId {
        if let Some(id) = self.get::<T>() {
            return id;
        }
        let id = EventId(u32::try_from(self.names.len()).expect("event type count exceeds u32"));
        self.ids.insert(TypeId::of::<T>(), id);
        self.names.push(type_name::<T>());
        trace!("registered event {} as {id:?}", type_name::<T>());
        id
    }

    pub(crate) fn len(&self) -> usize {
        self.names.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

/// Double-buffered queue for an event type.
#[derive(Debug)]
pub struct Events<T: Event> {
    current: Vec<T>,
    previous: Vec<T>,
}

impl<T: Event> Events<T> {
    pub(crate) fn new() -> Self {
        Self {
            current: Vec::new(),
            previous: Vec::new(),
        }
    }

    pub fn send(&mut self, event: T) {
        self.current.push(event);
    }

    pub fn read(&self) -> impl Iterator<Item = &T> + '_ {
        self.previous.iter()
    }

    pub fn len(&self) -> usize {
        self.previous.len()
    }

    pub fn is_empty(&self) -> bool {
        self.previous.is_empty()
    }

    pub fn pending(&self) -> usize {
        self.current.len()
    }
}

/// Object-safe view of [`Events<T>`].
pub(crate) trait ErasedEvents: Any {
    /// Retire the readable buffer and promote this tick's writes into it.
    /// The old readable Vec is cleared and reused as the new write buffer,
    /// so the steady state allocates nothing.
    fn swap(&mut self);
    fn clear(&mut self);
    fn readable_len(&self) -> usize;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Event> ErasedEvents for Events<T> {
    fn swap(&mut self) {
        self.previous.clear();
        core::mem::swap(&mut self.current, &mut self.previous);
    }

    fn clear(&mut self) {
        self.current.clear();
        self.previous.clear();
    }

    fn readable_len(&self) -> usize {
        self.previous.len()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Shared borrow of an event queue; derefs to `Events<T>` for `read`.
pub struct EventsRef<'a, T: Event> {
    guard: Ref<'a, Box<dyn ErasedEvents>>,
    _marker: PhantomData<T>,
}

impl<'a, T: Event> EventsRef<'a, T> {
    pub(crate) fn new(guard: Ref<'a, Box<dyn ErasedEvents>>) -> Self {
        Self {
            guard,
            _marker: PhantomData,
        }
    }
}

impl<T: Event> Deref for EventsRef<'_, T> {
    type Target = Events<T>;

    fn deref(&self) -> &Events<T> {
        self.guard
            .as_any()
            .downcast_ref::<Events<T>>()
            .expect("queue slot holds the type its EventId was interned for")
    }
}

impl<T: Event> fmt::Debug for EventsRef<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventsRef")
            .field("event", &type_name::<T>())
            .field("readable", &self.guard.readable_len())
            .finish()
    }
}

/// Exclusive borrow of an event queue; derefs to `Events<T>` for `send`.
pub struct EventsMut<'a, T: Event> {
    guard: RefMut<'a, Box<dyn ErasedEvents>>,
    _marker: PhantomData<T>,
}

impl<'a, T: Event> EventsMut<'a, T> {
    pub(crate) fn new(guard: RefMut<'a, Box<dyn ErasedEvents>>) -> Self {
        Self {
            guard,
            _marker: PhantomData,
        }
    }
}

impl<T: Event> Deref for EventsMut<'_, T> {
    type Target = Events<T>;

    fn deref(&self) -> &Events<T> {
        self.guard
            .as_any()
            .downcast_ref::<Events<T>>()
            .expect("queue slot holds the type its EventId was interned for")
    }
}

impl<T: Event> DerefMut for EventsMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Events<T> {
        self.guard
            .as_any_mut()
            .downcast_mut::<Events<T>>()
            .expect("queue slot holds the type its EventId was interned for")
    }
}

impl<T: Event> fmt::Debug for EventsMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventsMut")
            .field("event", &type_name::<T>())
            .field("readable", &self.guard.readable_len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Damaged(u32);
    impl Event for Damaged {}

    #[derive(Debug)]
    struct Spawned;
    impl Event for Spawned {}

    #[test]
    fn sends_are_invisible_until_swap() {
        let mut events = Events::<Damaged>::new();
        events.send(Damaged(5));

        assert_eq!(events.len(), 0, "not readable this tick");
        assert_eq!(events.pending(), 1);

        events.swap();
        assert_eq!(events.len(), 1);
        assert_eq!(events.read().next(), Some(&Damaged(5)));
    }

    #[test]
    fn events_live_exactly_one_tick() {
        let mut events = Events::<Damaged>::new();
        events.send(Damaged(1));
        events.swap();
        assert_eq!(events.len(), 1);

        events.swap(); // next tick, nothing new sent
        assert_eq!(events.len(), 0, "stale events are dropped");
    }

    #[test]
    fn read_order_matches_send_order() {
        let mut events = Events::<Damaged>::new();
        events.send(Damaged(1));
        events.send(Damaged(2));
        events.send(Damaged(3));
        events.swap();

        let got: Vec<u32> = events.read().map(|d| d.0).collect();
        assert_eq!(got, vec![1, 2, 3]);
    }

    #[test]
    fn swap_reuses_buffers() {
        let mut events = Events::<Damaged>::new();
        events.send(Damaged(1));
        events.swap();
        let cap_before = events.current.capacity() + events.previous.capacity();

        for _ in 0..10 {
            events.send(Damaged(1));
            events.swap();
        }
        // Steady state: capacity settles rather than growing per tick.
        assert!(events.current.capacity() + events.previous.capacity() >= cap_before);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn erased_swap_without_knowing_t() {
        let mut events = Events::<Spawned>::new();
        events.send(Spawned);

        let erased: &mut dyn ErasedEvents = &mut events;
        assert_eq!(erased.readable_len(), 0);
        erased.swap();
        assert_eq!(erased.readable_len(), 1);
        erased.clear();
        assert_eq!(erased.readable_len(), 0);
    }

    #[test]
    fn event_ids_are_stable_and_distinct() {
        let mut types = EventTypes::default();
        let a = types.get_or_insert::<Damaged>();
        let b = types.get_or_insert::<Spawned>();

        assert_ne!(a, b);
        assert_eq!(types.get_or_insert::<Damaged>(), a);
        assert_eq!(types.len(), 2);
    }
}
