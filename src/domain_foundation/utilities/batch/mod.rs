//! # Batch
//!
//! Optional operations over multiple objects.

pub mod operation;

pub use operation::{apply_batch, operation, FnBatchOperation, BatchOperation};