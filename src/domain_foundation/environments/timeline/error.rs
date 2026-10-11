//! Timeline-specific errors.

use super::clip::ClipId;
use super::keyframe::KeyframeId;
use super::track::TrackId;
use super::timeline::Time;
use std::fmt;

// Note: no `Eq` derive because `InvalidDuration` contains `f64`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum TimelineError {
    TrackNotFound { id: TrackId },
    ClipNotFound { id: ClipId },
    KeyframeNotFound { id: KeyframeId },
    /// A clip insertion or move would overlap an existing clip on the
    /// same track.
    ClipOverlap { id: ClipId, conflicting_with: ClipId },
    /// A clip with zero or negative duration was provided.
    InvalidDuration { seconds: f64 },

    InvalidTrackIndex { index: usize, track_count: usize },
    InvalidSplitPoint { time: Time },
}

impl fmt::Display for TimelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimelineError::TrackNotFound { id } => write!(f, "track {id} not found"),
            TimelineError::ClipNotFound { id } => write!(f, "clip {id} not found"),
            TimelineError::KeyframeNotFound { id } => write!(f, "keyframe {id} not found"),
            TimelineError::ClipOverlap { id, conflicting_with } => {
                write!(f, "clip {id} overlaps existing clip {conflicting_with}")
            }
            TimelineError::InvalidDuration { seconds } => {
                write!(f, "invalid duration: {seconds}s")
            }
            TimelineError::InvalidTrackIndex { index, track_count } => {
                write!(f, "track index {index} out of range (have {track_count} tracks)")
            }
            TimelineError::InvalidSplitPoint { time } => {
                write!(f, "invalid split point: {time}")
            }
        }
    }
}

impl std::error::Error for TimelineError {}