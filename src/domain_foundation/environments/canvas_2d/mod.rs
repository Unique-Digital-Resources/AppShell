//! # Canvas2D Environment
//!
//! Headless 2D spatial editing model. Provides object identity, transforms
//! and operations without imposing any UI layout, coordinate-system
//! orientation, or rendering.
//!
//! ## What this environment owns
//!
//! - Stable, opaque object identity via [`ObjectId`].
//! - Per-object spatial [`Transform`] (translation, rotation, scale).
//! - A [`Canvas`] container that maps ids to objects.
//! - Value-type [`CanvasOperation`]s describing *what* a change means, plus
//!   [`CanvasOutcome`]s carrying enough state to construct inverses later.
//!
//! ## What this environment deliberately does *not* own
//!
//! - Application-specific object data (color, kind, label). Domain Engines
//!   layer that externally, keyed by [`ObjectId`].
//! - Execution infrastructure (commands, history, events). Phase 4 will
//!   wrap [`CanvasOperation`]s in App Engine commands.
//! - Rendering, input, layout, animation, camera, viewport.
//!
//! ## Coordinate convention
//!
//! [`Vec2`] carries two `f64` components named `x` and `y`. The foundation
//! imposes no convention on orientation (y-up, y-down, screen pixels,
//! world units). Callers define their semantics.

pub mod canvas;
pub mod error;
pub mod object;
pub mod operations;
pub mod transform;

pub use canvas::Canvas;
pub use error::CanvasError;
pub use object::{Object, ObjectId};
pub use operations::{CanvasOperation, CanvasOutcome};
pub use transform::{Transform, Vec2};