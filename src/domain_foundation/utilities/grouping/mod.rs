//! # Grouping
//!
//! Optional grouping of objects by developer-defined keys.

pub mod criteria;

pub use criteria::{group, group_by, group_sorted, GroupCriteria};