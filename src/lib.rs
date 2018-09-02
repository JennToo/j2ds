//! A collection of misc. data structures that aren't available in the
//! standard library

mod clock;
mod ring;

pub use clock::{next_timer_event, Clock, Timer, TimerEvent};
pub use ring::{ElasticPopResult, ElasticRingBuffer, RingBuffer};
