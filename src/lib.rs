//! An easily embeddable, futures-friendly logic engine.
//!
//! If you want to understand more about this implementation, see
//! [How to replace failure by a list of successes](http://dl.acm.org/citation.cfm?id=5280.5288)
//! by Philip Wadler. This code does not follow that paper exactly (since we can encounter errors
//! during resolution), but the general approach is the same.

#![deny(missing_docs)]

#[macro_use]
extern crate failure;
extern crate frunk;
extern crate futures;
#[macro_use]
extern crate lalrpop_util;
#[macro_use]
extern crate lazy_static;
extern crate regex;

mod ast;
pub(crate) mod cst;
mod errors;
mod eval;
mod unify;
mod util;

#[cfg(test)]
mod tests;

lalrpop_mod!(grammar);

pub use ast::*;
pub use errors::{LoadError, ResolutionError};
pub use eval::Env;
