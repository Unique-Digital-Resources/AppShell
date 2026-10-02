//! Public boundary of the AppShell crate.
//!
//! External users access the App Engine through this module.

pub mod app_engine;

// Re-export the composition root at the crate level.
pub use app_engine::AppEngine;