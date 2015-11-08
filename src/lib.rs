#![cfg_attr(feature = "no_function_casts", feature(core, unboxed_closures))]

extern crate rand;
extern crate rand_distributions;

mod generate;
mod arbitrary;
mod call;
mod testable;
