//! # Filtering
//!
//! Evaluate objects against composable criteria.

pub mod criteria;
pub mod filter;

pub use criteria::{
    criteria, And, FilterCriteria, FnCriteria, Not, Or,
};
pub use filter::{all, any, count, filter};