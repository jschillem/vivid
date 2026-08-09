use std::time::{Duration, Instant};

use log::warn;

/// A fixed tick rate.
///
/// This essentially just serves to pre-compute the seconds as this value will likelyremain
/// constant for the duration of the app/game. If a new tick-rate is desired, its just a
/// one time calculation.
#[derive(Debug, Clone, Copy)]
pub struct TickRate {
    duration: Duration,
    seconds: f32,
}

impl TickRate {
    /// Crates a tick rate based off of a clock `Hz`.
    ///
    /// # Panics
    /// If `hz` is zero.
    pub fn new(hz: u32) -> Self {
        assert!(hz > 0, "tick rate must be non-zero");
        let duration = Duration::from_secs(1) / hz;
        Self {
            duration,
            seconds: duration.as_secs_f32(),
        }
    }

    #[inline]
    pub const fn duration(&self) -> Duration {
        self.duration
    }

    #[inline]
    pub const fn seconds(&self) -> f32 {
        self.seconds
    }
}

/// A game clock.
#[derive(Debug)]
pub struct Clock {
    tick: TickRate,
    accumulator: Duration,
    last: Instant,
    /// Last frame's (clamped) elapsed time. i.e. the variable frame delta.
    frame: Duration,
    /// Per-frame elapsed-time clamp. Prevents pausing from queuing
    /// extra catch-up ticks.
    max_frame: Duration,
}

impl Clock {
    /// A clock ticking `hz` times per simulated second.
    ///
    /// # Panics
    /// If `hz` is zero.
    pub fn new(hz: u32) -> Self {
        Self {
            tick: TickRate::new(hz),
            accumulator: Duration::ZERO,
            last: Instant::now(),
            frame: Duration::ZERO,
            max_frame: Duration::from_millis(250),
        }
    }

    /// The fixed tick duration in seconds.
    pub fn tick_seconds(&self) -> f32 {
        self.tick.seconds
    }

    /// Last frame's e(clamped) lapsed seconds.
    ///
    /// NOTE: This should **NOT** be used for simulation logic; use [`tick_seconds`] instead.
    pub fn frame_seconds(&self) -> f32 {
        self.frame.as_secs_f32()
    }

    /// Called exactly once per render frame.
    ///
    /// Measures the real elapsed time and returns how many sim ticks to run right now.
    pub fn advance(&mut self) -> u32 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last);
        self.last = now;
        self.advance_by(elapsed)
    }

    /// Advances the clock by a specified [`Duration`].
    pub fn advance_by(&mut self, mut elapsed: Duration) -> u32 {
        if elapsed > self.max_frame {
            warn!("frame took {elapsed:?}; clamping to {:?}", self.max_frame);
            elapsed = self.max_frame;
        }

        self.frame = elapsed;
        self.accumulator += elapsed;

        let mut ticks = 0;
        while self.accumulator >= self.tick.duration {
            self.accumulator -= self.tick.duration;
            ticks += 1;
        }
        ticks
    }

    /// Fraction of a tick the current render moment sits past the latest sim state.
    ///
    /// in the range `[0,1)`.
    ///
    pub fn alpha(&self) -> f32 {
        self.accumulator.as_secs_f32() / self.tick.seconds
    }
}

#[cfg(test)]
mod tests {
    use approx::{assert_abs_diff_eq, assert_relative_eq};

    use super::*;

    #[test]
    fn accumulates_and_drains_whole_ticks() {
        let mut clock = Clock::new(60);
        // 50 ms at 60 Hz (16.66 ms ticks) = 3 whole ticks + remainder.
        let ticks = clock.advance_by(Duration::from_millis(50));
        assert_eq!(ticks, 3);
        assert_abs_diff_eq!(clock.alpha(), 0.0, epsilon = 1e-4);
    }

    #[test]
    fn sub_tick_frames_run_zero_ticks_but_grow_alpha() {
        let mut clock = Clock::new(60);
        assert_eq!(clock.advance_by(Duration::from_millis(10)), 0);
        let a1 = clock.alpha();
        assert_eq!(clock.advance_by(Duration::from_millis(5)), 0);
        assert!(clock.alpha() > a1, "alpha accumulates across short frames");
    }

    #[test]
    fn huge_frame_is_clamped() {
        let mut clock = Clock::new(60);
        // 10 s unclamped would be 600 ticks; the 250 ms clamp allows 15.
        assert_eq!(clock.advance_by(Duration::from_secs(10)), 15);
    }

    #[test]
    fn leftover_carries_between_frames() {
        let mut clock = Clock::new(100); // 10 ms ticks
        assert_eq!(clock.advance_by(Duration::from_millis(9)), 0);
        assert_eq!(
            clock.advance_by(Duration::from_millis(2)),
            1,
            "9+2 crosses one tick"
        );
    }

    #[test]
    fn frame_seconds_reports_clamped_elapsed() {
        let mut clock = Clock::new(60);

        // relative just to avoid the lint, but epsilon = 0.0 fixes that
        assert_relative_eq!(clock.frame_seconds(), 0.0_f32, epsilon = 0.);

        clock.advance_by(Duration::from_millis(20));
        assert_abs_diff_eq!(clock.frame_seconds(), 0.020, epsilon = 1e-6);

        // The frame delta is clamped like the accumulator input: a stall
        // must not hand render-stage animation a giant dt either.
        clock.advance_by(Duration::from_secs(10));
        assert_abs_diff_eq!(clock.frame_seconds(), 0.250, epsilon = 1e-6);
    }
}
