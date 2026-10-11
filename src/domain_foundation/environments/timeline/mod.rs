//! # Timeline Environment
//!
//! Headless temporal editing model. Provides tracks, clips, keyframes and
//! time-based operations without imposing any UI, rendering, or playback
//! engine.
//!
//! ## What this environment owns
//!
//! - [`Time`] and [`Duration`] types for temporal positioning.
//! - [`Track`]s as independent containers for clips.
//! - [`Clip`]s as time-bounded segments placed on tracks.
//! - [`Keyframe`]s for animated values at specific times.
//! - [`TimelineOperation`]s as value-type intents with undoable outcomes.
//!
//! ## What this environment deliberately does *not* own
//!
//! - Playback engine, clock, or frame-rate logic.
//! - Media decoding, audio rendering, or video frame extraction.
//! - Application-specific clip semantics (what the clip *contains*).
//! - Temporal dependency rules — a Domain Engine can define those using
//!   the Phase 3 [`relations`](crate::domain_foundation::relations) system.
//!
//! ## No shared base with Canvas2D
//!
//! Timeline shares no base trait or common model with Canvas2D. Each owns
//! its own types, errors, and operations. The only shared concept is the
//! documentation convention of "an environment with operations."

pub mod clip;
pub mod error;
pub mod keyframe;
pub mod operations;
pub mod timeline;
pub mod track;

pub use clip::{Clip, ClipId};
pub use error::TimelineError;
pub use keyframe::{Interpolation, Keyframe, KeyframeId, KeyframeValue};
pub use operations::{TimelineOperation, TimelineOutcome};
pub use timeline::{Duration, Time, Timeline};
pub use track::{Track, TrackId};