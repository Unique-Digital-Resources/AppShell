//! Track model: an independent container for an ordered sequence of clips.

use super::clip::{Clip, ClipId};
use super::error::TimelineError;
use super::timeline::Time;
use std::fmt;

/// Stable, opaque identifier for a track within a [`Timeline`](super::Timeline).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TrackId(u64);

impl TrackId {
    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub fn as_raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for TrackId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "track{}", self.0)
    }
}

/// A track: an independent timeline lane containing an ordered sequence
/// of non-overlapping [`Clip`]s.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub id: TrackId,
    clips: Vec<Clip>,
}

impl Track {
    pub fn new(id: TrackId) -> Self {
        Self { id, clips: Vec::new() }
    }

    /// Insert a clip, maintaining sort order by start time. Rejects
    /// overlaps with existing clips on this track.
    pub fn insert_clip(&mut self, clip: Clip) -> Result<(), TimelineError> {
        for existing in &self.clips {
            if clip.overlaps(existing) {
                return Err(TimelineError::ClipOverlap {
                    id: clip.id,
                    conflicting_with: existing.id,
                });
            }
        }
        let pos = self
            .clips
            .partition_point(|c| c.start < clip.start);
        self.clips.insert(pos, clip);
        Ok(())
    }

    pub fn remove_clip(&mut self, id: ClipId) -> Result<Clip, TimelineError> {
        let pos = self
            .clips
            .iter()
            .position(|c| c.id == id)
            .ok_or(TimelineError::ClipNotFound { id })?;
        Ok(self.clips.remove(pos))
    }

    pub fn clip(&self, id: ClipId) -> Option<&Clip> {
        self.clips.iter().find(|c| c.id == id)
    }

    pub fn clip_mut(&mut self, id: ClipId) -> Option<&mut Clip> {
        self.clips.iter_mut().find(|c| c.id == id)
    }

    pub fn clips(&self) -> &[Clip] {
        &self.clips
    }

    pub fn clip_count(&self) -> usize {
        self.clips.len()
    }

    pub fn is_empty(&self) -> bool {
        self.clips.is_empty()
    }

    /// Move a clip to a new start time. Temporarily removes the clip,
    /// checks for overlap at the new position, and re-inserts if safe.
    /// Returns the previous start time.
    pub fn move_clip(&mut self, id: ClipId, new_start: Time) -> Result<Time, TimelineError> {
        let clip = self.remove_clip(id)?;
        let previous_start = clip.start;

        // Clone the clip and modify the copy for overlap checking.
        let mut moved = clip.clone();
        moved.start = new_start;

        // Check for overlap at the new position. The removed clip is no
        // longer in self.clips, so it won't conflict with itself.
        // Collect the conflict before mutating to avoid borrow issues.
        let conflict = self
            .clips
            .iter()
            .find(|existing| moved.overlaps(existing))
            .map(|c| c.id);

        if let Some(conflicting_with) = conflict {
            // Re-insert at original position and return error.
            self.insert_clip(clip)?;
            return Err(TimelineError::ClipOverlap {
                id,
                conflicting_with,
            });
        }

        self.insert_clip(moved)?;
        Ok(previous_start)
    }
    /// Replace a clip (used by split — remove old, insert left + right).
    pub fn replace_clip(&mut self, old_id: super::clip::ClipId, clips: Vec<super::clip::Clip>)
        -> Result<(), super::error::TimelineError>
    {
        // Remove the old clip.
        self.remove_clip(old_id)?;
        // Insert the new clips.
        for clip in clips {
            self.insert_clip(clip)?;
        }
        Ok(())
    }
}