use std::fmt;

use log::{debug, trace};
use vivid_core::World;
use vivid_platform::KeyboardInput;
use vivid_render::Camera;

/// Context for use in simulation systems.
/// This gives context to the engine's tick system that runs on a fixed interval.
#[derive(Debug)]
pub struct TickCtx<'a> {
    pub world: &'a mut World,
    pub input: &'a KeyboardInput,
    pub delta_time: f32,
    pub(crate) quit: bool,
}

impl TickCtx<'_> {
    /// Inform the engine to shut down after the current frame completes.
    pub fn request_quit(&mut self) {
        self.quit = true;
    }
}

#[derive(Debug)]
pub struct FrameCtx<'a> {
    pub world: &'a mut World,
    pub input: &'a KeyboardInput,
    pub camera: &'a Camera,
    pub delta_time: f32,
    pub alpha: f32,
    pub(crate) quit: bool,
}

impl FrameCtx<'_> {
    /// Inform the engine to shut down after the current frame completes.
    pub fn request_quit(&mut self) {
        self.quit = true;
    }
}

/// A game system.
struct System<F> {
    name: &'static str,
    run: F,
}

/// An ordered list of systems.
pub struct Schedule<F> {
    systems: Vec<System<F>>,
}

/// Systems that run on the fixed simulation tick.
pub type TickSchedule = Schedule<Box<dyn FnMut(&mut TickCtx<'_>)>>;
/// Systems that run once per rendered frame.
pub type FrameSchedule = Schedule<Box<dyn FnMut(&mut FrameCtx<'_>)>>;

impl<F> Schedule<F> {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    /// Add a new system to the schedule.
    pub fn push(&mut self, name: &'static str, run: F) -> &mut Self {
        debug!(
            "scheduled system '{name}' at position {}",
            self.systems.len()
        );
        self.systems.push(System { name, run });
        self
    }

    pub fn names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.systems.iter().map(|s| s.name)
    }

    pub fn len(&self) -> usize {
        self.systems.len()
    }

    pub fn is_empty(&self) -> bool {
        self.systems.is_empty()
    }
}

impl TickSchedule {
    /// Append a simulation system.
    pub fn add<F>(&mut self, name: &'static str, system: F) -> &mut Self
    where
        F: FnMut(&mut TickCtx<'_>) + 'static,
    {
        self.push(name, Box::new(system))
    }

    /// Run every system once, in order.
    pub fn run(&mut self, ctx: &mut TickCtx<'_>) {
        for system in &mut self.systems {
            trace!("running '{}'", system.name);
            (system.run)(ctx);
        }
    }
}

impl FrameSchedule {
    /// Append a presentation system.
    pub fn add<F>(&mut self, name: &'static str, system: F) -> &mut Self
    where
        F: FnMut(&mut FrameCtx<'_>) + 'static,
    {
        self.push(name, Box::new(system))
    }

    pub fn run(&mut self, ctx: &mut FrameCtx<'_>) {
        for system in &mut self.systems {
            trace!("running '{}'", system.name);
            (system.run)(ctx);
        }
    }
}

impl<C> Default for Schedule<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> fmt::Debug for Schedule<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Schedule")
            .field("systems", &self.names().collect::<Vec<_>>())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Minimal stand-in context: these tests are about ordering and
    /// bookkeeping, not about worlds. Boxed the same way real schedules are.
    struct TestCtx {
        log: Rc<RefCell<Vec<&'static str>>>,
        counter: u32,
    }

    type TestSchedule = Schedule<Box<dyn FnMut(&mut TestCtx)>>;

    fn ctx(log: &Rc<RefCell<Vec<&'static str>>>) -> TestCtx {
        TestCtx {
            log: Rc::clone(log),
            counter: 0,
        }
    }

    fn record(name: &'static str) -> Box<dyn FnMut(&mut TestCtx)> {
        Box::new(move |c: &mut TestCtx| c.log.borrow_mut().push(name))
    }

    #[test]
    fn runs_in_registration_order() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut schedule = TestSchedule::new();
        schedule.push("first", record("first"));
        schedule.push("second", record("second"));
        schedule.push("third", record("third"));

        let mut c = ctx(&log);
        for system in &mut schedule.systems {
            (system.run)(&mut c);
        }

        assert_eq!(*log.borrow(), vec!["first", "second", "third"]);
    }

    #[test]
    fn systems_keep_their_own_state_across_runs() {
        // FnMut, not Fn: a system may own mutable state (timers, caches).
        let mut schedule = TestSchedule::new();
        let mut calls = 0;
        schedule.push(
            "stateful",
            Box::new(move |c: &mut TestCtx| {
                calls += 1;
                c.counter = calls;
            }),
        );

        let log = Rc::new(RefCell::new(Vec::new()));
        let mut c = ctx(&log);
        for _ in 0..3 {
            for system in &mut schedule.systems {
                (system.run)(&mut c);
            }
        }

        assert_eq!(c.counter, 3);
    }

    #[test]
    fn names_reflect_the_pipeline() {
        let mut schedule = TestSchedule::new();
        schedule.push("integrate", record("integrate"));
        schedule.push("collide", record("collide"));

        assert_eq!(
            schedule.names().collect::<Vec<_>>(),
            vec!["integrate", "collide"]
        );
        assert_eq!(schedule.len(), 2);
    }

    #[test]
    fn empty_schedule_is_empty() {
        let schedule = TestSchedule::new();
        assert!(schedule.is_empty());
        assert_eq!(schedule.names().count(), 0);
    }

    #[test]
    fn debug_prints_the_pipeline() {
        let mut schedule = TestSchedule::new();
        schedule.push("integrate", record("integrate"));
        assert!(format!("{schedule:?}").contains("integrate"));
    }
}
