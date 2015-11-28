#![cfg_attr(feature = "no_function_casts", feature(core, unboxed_closures))]

extern crate rand;
extern crate num;

extern crate env_logger;
#[macro_use] extern crate log;

#[macro_use]
mod macros;

pub mod generate;
pub mod arbitrary;
mod quick_fn;
pub mod rose;
pub mod property;
pub mod testable;
pub mod quick_check;

pub use quick_check::{quickcheck, quicktest, QuickCheck};

#[cfg(test)]
mod tests;
