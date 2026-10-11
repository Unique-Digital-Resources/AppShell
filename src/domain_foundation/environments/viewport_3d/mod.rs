//! # Viewport3D Environment
//!
//! Headless 3D spatial editing model. Provides objects, transforms and
//! operations without imposing any UI, rendering engine, camera, or
//! projection.
//!
//! ## What this environment owns
//!
//! - [`Vec3`] for 3D positions, directions and Euler-angle rotations.
//! - [`Transform3D`] with translation, rotation (pitch/yaw/roll) and scale.
//! - [`Object3D`] identity and spatial state.
//! - [`Scene`] container with stable [`ObjectId3D`]s.
//! - [`Viewport3DOperation`]s as value-type intents with undoable outcomes.
//!
//! ## What this environment deliberately does *not* own
//!
//! - Camera, projection, viewport, or frustum logic.
//! - Rendering, mesh data, materials, or shaders.
//! - Quaternion rotation (Euler angles suffice for a headless model;
//!   Domain Engines can add quaternion support if needed).
//! - Application-specific object data (mesh references, textures).
//!
//! ## No shared base with Canvas2D
//!
//! Viewport3D shares no types with Canvas2D. `Vec3` is separate from
//! `Vec2`. The two environments are siblings, each owning their own
//! spatial primitives.

pub mod error;
pub mod object;
pub mod operations;
pub mod scene;
pub mod transform;

pub use error::Viewport3DError;
pub use object::{Object3D, ObjectId3D};
pub use operations::{Viewport3DOperation, Viewport3DOutcome};
pub use scene::Scene;
pub use transform::{Transform3D, Vec3};