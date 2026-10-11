//! Clip model: a time-bounded segment on a track, with keyframes.

use super::keyframe::{Keyframe, KeyframeId, KeyframeValue};
use super::timeline::{Duration, Time};
use super::track::TrackId;
use std::fmt;

/// Stable identifier for a clip. Carries its track so the timeline can
/// find the right track without a separate index.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClipId {
    pub raw: u64,
    pub track: TrackId,
}

impl ClipId {
    pub fn from_raw(raw: u64, track: TrackId) -> Self {
        Self { raw, track }
    }
}

impl fmt::Display for ClipId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "clip{}@{}", self.raw, self.track)
    }
}

/// A time-bounded segment placed on a track.
///
/// The foundation does not attach media-specific data to clips. A Domain
/// Engine layers content (audio, video, text, etc.) externally, keyed by
/// [`ClipId`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Clip {
    pub id: ClipId,
    pub track: TrackId,
    pub start: Time,
    pub duration: Duration,
    keyframes: Vec<Keyframe>,
}

impl Clip {
    pub fn new(id: ClipId, track: TrackId, start: Time, duration: Duration) -> Self {
        Self {
            id,
            track,
            start,
            duration,
            keyframes: Vec::new(),
        }
    }

    pub fn end(&self) -> Time {
        Time(self.start.0 + self.duration.0)
    }

    pub fn keyframes(&self) -> &[Keyframe] {
        &self.keyframes
    }

    pub fn add_keyframe(&mut self, kf: Keyframe) {
        let pos = self
            .keyframes
            .partition_point(|k| k.time < kf.time);
        self.keyframes.insert(pos, kf);
    }

    pub fn remove_keyframe(&mut self, id: KeyframeId) -> Result<Keyframe, super::error::TimelineError> {
        let pos = self
            .keyframes
            .iter()
            .position(|k| k.id == id)
            .ok_or(super::error::TimelineError::KeyframeNotFound { id })?;
        Ok(self.keyframes.remove(pos))
    }

    /// Check whether this clip's time interval overlaps another's.
    pub fn overlaps(&self, other: &Clip) -> bool {
        let self_end = self.start.0 + self.duration.0;
        let other_end = other.start.0 + other.duration.0;
        self.start.0 < other_end && other.start.0 < self_end
    }

    /// All keyframes whose time falls within `[start, end)`.
    pub fn keyframes_in_range(&self, start: Time, end: Time) -> Vec<&Keyframe> {
        self.keyframes
            .iter()
            .filter(|kf| {
                kf.time.seconds() >= start.seconds() && kf.time.seconds() < end.seconds()
            })
            .collect()
    }

    /// Whether `time` falls within this clip's span `[start, start + duration)`.
    pub fn contains(&self, time: Time) -> bool {
        let start = self.start.seconds();
        let end = start + self.duration.seconds();
        time.seconds() >= start && time.seconds() < end
    }

    /// Set the clip's duration without changing its start time.
    pub fn set_duration(&mut self, duration: super::timeline::Duration) {
        self.duration = duration;
    }

    /// Split this clip into two at `at_time`. Returns the two clips
    /// (left, right). The left clip keeps the original id; the right
    /// gets `new_id`.
    pub fn split(self, new_id: super::clip::ClipId, at_time: super::timeline::Time)
        -> Result<(Self, Self), super::error::TimelineError>
    {
        let at = at_time.seconds();
        let start = self.start.seconds();
        let end = start + self.duration.seconds();

        if at <= start || at >= end {
            return Err(super::error::TimelineError::InvalidSplitPoint { time: at_time });
        }

        let left_duration = super::timeline::Duration::new(at - start);
        let right_duration = super::timeline::Duration::new(end - at);

        let mut left = self;
        left.duration = left_duration;

        let right = Clip::new(new_id, left.track, at_time, right_duration);

        Ok((left, right))
    }

    pub fn keyframes_mut(&mut self) -> &mut [Keyframe] {
        &mut self.keyframes
    }


}