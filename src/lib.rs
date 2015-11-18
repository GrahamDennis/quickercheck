#![cfg_attr(feature = "no_function_casts", feature(core, unboxed_closures))]

extern crate rand;
extern crate rand_distributions;
extern crate num;

extern crate env_logger;
#[macro_use] extern crate log;

#[macro_use]
mod macros;

mod generate;
mod arbitrary;
mod quick_fn;
mod property;
mod testable;
mod quick_check;

#[cfg(test)]
mod tests;
