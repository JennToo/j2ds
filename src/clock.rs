/// An increasing counter that ticks up until a particular count is
/// reached, which then resets itself
///
/// Example:
///
/// ```rust
/// use j2ds::*;
///
/// fn periodically_called_function(clock: &mut Clock) {
///     // Do some stuff...
///
///     if clock.tick() {
///         // Do something special...
///     }
/// }
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct Clock {
    count: u64,
    period: u64,
}

impl Clock {
    /// Create a new clock that cycles every `period` ticks
    pub fn new(period: u64) -> Clock {
        Clock {
            count: 0,
            period: period,
        }
    }

    /// Increment the current count by 1. If this is the `period`-th
    /// tick, the counter is reset and `true` is returned.
    pub fn tick(&mut self) -> bool {
        self.count += 1;
        assert!(self.count <= self.period);
        if self.count >= self.period {
            self.count = 0;
            true
        } else {
            false
        }
    }

    /// Reset the current count
    pub fn reset(&mut self) {
        self.count = 0;
    }

    /// Return the current count
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Return the period of the clock
    pub fn period(&self) -> u64 {
        self.period
    }
}

#[test]
fn test_clock() {
    let mut c = Clock::new(3);
    assert_eq!(c.period(), 3);

    // First round
    assert_eq!(c.count(), 0);
    assert!(!c.tick());
    assert_eq!(c.count(), 1);
    assert!(!c.tick());
    assert_eq!(c.count(), 2);
    assert!(c.tick());
    assert_eq!(c.count(), 0);

    // Wrap and second round
    assert!(!c.tick());
    assert!(!c.tick());
    assert!(c.tick());

    // Reset in the middle of a cycle
    assert!(!c.tick());
    c.reset();
    assert!(!c.tick());
    assert!(!c.tick());
    assert!(c.tick());
}

/// A periodic timer with rising and falling edges
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Timer {
    period: u64,
    next_start: u64,
    next_stop: u64,
}

/// Indicates which edge of the timer was just hit
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum TimerEvent {
    RisingEdge,
    FallingEdge,
}

impl Timer {
    /// Creaste a new timer that activates every `period` ticks,
    /// starts at the given `offset` timer, and lasts for `duration`
    /// ticks. The offset and duration must be less than the
    /// period. The duration may be 0, and in that case the timer will
    /// only emit `RisingEdge` events
    pub fn new(period: u64, offset: u64, duration: u64) -> Timer {
        assert!(offset < period);
        assert!(duration < period);

        Timer {
            period,
            next_start: offset,
            next_stop: offset + duration,
        }
    }

    /// Get the next tick that will emit a `RisingEdge` event
    pub fn next_start_time(&self) -> u64 {
        self.next_start
    }

    /// Get the next tick that will emit a `FallingEdge` event
    pub fn next_stop_time(&self) -> u64 {
        self.next_stop
    }

    /// Get the next tick that will produce any `TimerEvent`
    pub fn next_event_time(&self) -> u64 {
        if self.next_start < self.next_stop {
            self.next_start
        } else {
            self.next_stop
        }
    }

    /// Runs the timer until either the given absolute `time` is
    /// reached, or until the next event occurs. You should generally
    /// run this function in a loop, as multiple events may have
    /// occured in the time elapsed.
    pub fn update(&mut self, time: u64) -> Option<TimerEvent> {
        if self.next_start <= self.next_stop && self.next_start <= time {
            if self.next_stop == self.next_start {
                self.next_stop += self.period;
            }
            self.next_start += self.period;
            Some(TimerEvent::RisingEdge)
        } else if self.next_stop <= time {
            self.next_stop += self.period;
            Some(TimerEvent::FallingEdge)
        } else {
            None
        }
    }

    /// Indicates if the timer is currently between a `RisingEdge` and
    /// `FallingEdge` event
    pub fn is_active(&self) -> bool {
        self.next_start > self.next_stop
    }
}

/// Given a list of `timers`, return the next tick that any of the
/// timers will emit a `TimerEvent`
pub fn next_timer_event(timers: &[Timer]) -> u64 {
    timers
        .iter()
        .map(|t| t.next_event_time())
        .min()
        .unwrap_or(0)
}

#[test]
fn test_timer() {
    let mut timer = Timer::new(100, 13, 20);

    assert_eq!(timer.next_start_time(), 13);
    assert_eq!(timer.next_stop_time(), 13 + 20);

    // Come up to just before the next start time
    assert_eq!(timer.update(12), None);
    assert_eq!(timer.next_start_time(), 13);
    assert_eq!(timer.next_stop_time(), 13 + 20);

    assert_eq!(timer.update(13), Some(TimerEvent::RisingEdge));
    assert_eq!(timer.next_start_time(), 13 + 100);
    assert_eq!(timer.next_stop_time(), 13 + 20);

    // Overshooting should still get the falling edge event
    assert_eq!(timer.update(13 + 20 + 5), Some(TimerEvent::FallingEdge));
    assert_eq!(timer.next_start_time(), 13 + 100);
    assert_eq!(timer.next_stop_time(), 13 + 20 + 100);

    let mut v = vec![];
    while let Some(e) = timer.update(300) {
        v.push(e);
    }
    // Ensure the events are interleaved properly when there are
    // several pending
    assert_eq!(
        v,
        vec![
            TimerEvent::RisingEdge,
            TimerEvent::FallingEdge,
            TimerEvent::RisingEdge,
            TimerEvent::FallingEdge,
        ]
    );
}

#[test]
fn test_timer_zero_duration() {
    let mut timer = Timer::new(100, 13, 0);

    assert_eq!(timer.next_start_time(), 13);
    assert_eq!(timer.update(12), None);
    assert_eq!(timer.next_start_time(), 13);

    assert_eq!(timer.update(13), Some(TimerEvent::RisingEdge));
    assert_eq!(timer.next_start_time(), 13 + 100);

    assert_eq!(timer.update(13 + 100), Some(TimerEvent::RisingEdge));
}

#[test]
fn test_next_timer_event() {
    let t1 = Timer::new(100, 13, 0);
    let t2 = Timer::new(100, 14, 0);

    assert_eq!(next_timer_event(&[t1, t2]), 13);
}
