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
}

#[test]
fn test_clock() {
    let mut c = Clock::new(3);

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
