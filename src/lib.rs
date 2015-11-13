#![cfg_attr(feature = "no_function_casts", feature(core, unboxed_closures))]

extern crate rand;
extern crate rand_distributions;

#[macro_use]
mod macros;

mod generate;
mod arbitrary;
mod quick_fn;
mod property;
mod testable;
mod quick_check;
