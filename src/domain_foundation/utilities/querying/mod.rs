//! # Querying
//!
//! Named, reusable queries built on the [`filtering`](super::filtering)
//! utility.

pub mod evaluator;
pub mod query;

pub use evaluator::evaluate;
pub use query::Query;