//! # Sorting
//!
//! Optional ordering by developer-defined criteria.

pub mod criteria;

pub use criteria::{by, sort, sorted, FnSortCriteria, SortCriteria};