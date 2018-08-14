//! A collection of misc. data structures that aren't available in the
//! standard library

mod clock;
mod ring;

pub use clock::Clock;
pub use ring::{ElasticPopResult, ElasticRingBuffer, RingBuffer};
