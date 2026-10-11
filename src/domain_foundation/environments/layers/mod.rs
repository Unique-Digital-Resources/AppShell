//! # Layers Environment
//!
//! Headless hierarchy model. Provides layers with parent-child
//! relationships, sibling ordering, visibility and lock state — without
//! imposing any UI, compositing engine, or rendering.
//!
//! ## What this environment owns
//!
//! - [`LayerId`] and [`Layer`] identity and properties.
//! - [`Layers`] hierarchy with parent-child and sibling ordering.
//! - [`LayersOperation`]s as value-type intents with undoable outcomes.
//!
//! ## What this environment deliberately does *not* own
//!
//! - Compositing properties (opacity, blend mode, masks). Domain Engines
//!   layer those externally.
//! - Rendering, visual panels, drag-and-drop.
//! - Hierarchy *relations* — a Domain Engine can define semantic
//!   dependencies using the Phase 3 [`relations`](crate::domain_foundation::relations)
//!   system. The Layers environment manages its own structural hierarchy.
//!
//! ## No shared base with other environments
//!
//! Layers shares no base trait with Canvas2D or Timeline. Each owns its
//! own types, errors, and operations.

pub mod error;
pub mod hierarchy;
pub mod layer;
pub mod operations;

pub use error::LayersError;
pub use hierarchy::Layers;
pub use layer::{Layer, LayerId};
pub use operations::{LayersOperation, LayersOutcome};