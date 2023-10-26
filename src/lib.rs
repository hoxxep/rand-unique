#![doc = include_str!("../README.md")]
#![no_std]
#[cfg(test)]
extern crate std;

#[doc(inline)]
pub use crate::builder::RandomSequenceBuilder;
#[doc(inline)]
pub use crate::sequence::RandomSequence;

mod builder;
#[cfg(feature = "rand")]
mod rand;
mod seed;
mod sequence;
