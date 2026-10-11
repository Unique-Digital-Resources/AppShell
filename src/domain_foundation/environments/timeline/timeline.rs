//! Timeline model: owns tracks, issues ids, and provides queries.
//!
//! Also defines [`Time`] and [`Duration`] — the temporal primitives used
//! throughout this environment.

use super::clip::{Clip, ClipId};
use super::error::TimelineError;
use super::keyframe::{Keyframe, KeyframeId, KeyframeValue};
use super::track::{Track, TrackId};
use std::collections::HashMap;

/// Absolute time position, in seconds.
///
/// The foundation does not enforce a specific unit, but all Timeline
/// operations use seconds by convention. `Time` is a newtype over `f64`
/// to prevent accidental mixing with [`Duration`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Time(pub f64);

impl Time {
    pub const fn new(seconds: f64) -> Self {
        Self(seconds)
    }

    pub const fn seconds(self) -> f64 {
        self.0
    }

    pub const ZERO: Self = Self(0.0);
}

impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3}s", self.0)
    }
}

impl From<f64> for Time {
    fn from(v: f64) -> Self {
        Self(v)
    }
}

/// A relative duration, in seconds.
///
/// Distinct from [`Time`] (absolute position) to prevent accidental
/// mixing of "when" and "how long".
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Duration(pub f64);

impl Duration {
    pub const fn new(seconds: f64) -> Self {
        Self(seconds)
    }

    pub const fn seconds(self) -> f64 {
        self.0
    }

    pub const ZERO: Self = Self(0.0);
}

impl std::fmt::Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3}s", self.0)
    }
}

impl From<f64> for Duration {
    fn from(v: f64) -> Self {
        Self(v)
    }
}

/// Headless temporal editing model.
///
/// A timeline owns a set of [`Track`]s, each of which contains an
/// ordered sequence of [`Clip`]s. Tracks are independent; clips within
/// a track may not overlap.
///
/// The timeline issues stable [`TrackId`]s, [`ClipId`]s, and
/// [`KeyframeId`]s. Ids are not reused after removal.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Timeline {
    tracks: HashMap<TrackId, Track>,
    track_order: Vec<TrackId>,
    next_track_id: u64,
    next_clip_id: u64,
    next_keyframe_id: u64,
}

impl Timeline {
    pub fn new() -> Self {
        Self::default()
    }

    // --- Track management ---

    pub fn create_track(&mut self) -> TrackId {
        let id = TrackId::from_raw(self.next_track_id);
        self.next_track_id += 1;
        self.tracks.insert(id, Track::new(id));
        self.track_order.push(id);
        id
    }

    /// Insert an existing track (for undo of RemoveTrack). Restores the
    /// track with its original id and all its clips.
    pub fn insert_track(&mut self, track: Track) -> TrackId {
        let id = track.id;
        if id.as_raw() >= self.next_track_id {
            self.next_track_id = id.as_raw() + 1;
        }
        if !self.track_order.contains(&id) {
            self.track_order.push(id);
        }
        self.tracks.insert(id, track);
        id
    }

    pub fn remove_track(&mut self, id: TrackId) -> Result<Track, TimelineError> {
        let track = self.tracks
            .remove(&id)
            .ok_or(TimelineError::TrackNotFound { id })?;
        self.track_order.retain(|t| *t != id);
        Ok(track)
    }

    pub fn track(&self, id: TrackId) -> Option<&Track> {
        self.tracks.get(&id)
    }

    pub fn track_mut(&mut self, id: TrackId) -> Option<&mut Track> {
        self.tracks.get_mut(&id)
    }

    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    pub fn tracks(&self) -> impl Iterator<Item = &Track> {
        self.track_order
            .iter()
            .filter_map(|id| self.tracks.get(id))
    }

    pub fn contains_track(&self, id: TrackId) -> bool {
        self.tracks.contains_key(&id)
    }

    /// Track ids in creation/display order.
    pub fn track_ids(&self) -> &[TrackId] {
        &self.track_order
    }

    /// Reorder a track within the track list. Returns the previous index.
    pub fn move_track(&mut self, id: TrackId, new_index: usize) -> Result<usize, TimelineError> {
        if !self.tracks.contains_key(&id) {
            return Err(TimelineError::TrackNotFound { id });
        }
        let current = self
            .track_order
            .iter()
            .position(|t| *t == id)
            .ok_or(TimelineError::TrackNotFound { id })?;
        if new_index >= self.track_order.len() {
            return Err(TimelineError::InvalidTrackIndex {
                index: new_index,
                track_count: self.track_order.len(),
            });
        }
        if current == new_index {
            return Ok(current);
        }
        self.track_order.remove(current);
        self.track_order.insert(new_index, id);
        Ok(current)
    }




    // --- Clip management ---

    pub fn insert_clip(
        &mut self,
        track_id: TrackId,
        start: Time,
        duration: Duration,
    ) -> Result<ClipId, TimelineError> {
        let next_id = ClipId::from_raw(self.next_clip_id, track_id);
        self.next_clip_id += 1;

        let track = self
            .tracks
            .get_mut(&track_id)
            .ok_or(TimelineError::TrackNotFound { id: track_id })?;

        track.insert_clip(Clip::new(next_id, track_id, start, duration))?;
        Ok(next_id)
    }

    /// Re-insert a clip (for undo of RemoveClip). The clip carries its
    /// original id and track.
    pub fn reinsert_clip(&mut self, clip: Clip) -> Result<ClipId, TimelineError> {
        let id = clip.id;
        let track_id = clip.track;
        let track = self
            .tracks
            .get_mut(&track_id)
            .ok_or(TimelineError::TrackNotFound { id: track_id })?;
        track.insert_clip(clip)?;
        Ok(id)
    }

    pub fn remove_clip(&mut self, id: ClipId) -> Result<Clip, TimelineError> {
        let track = self
            .tracks
            .get_mut(&id.track)
            .ok_or(TimelineError::TrackNotFound { id: id.track })?;
        track.remove_clip(id)
    }

    pub fn clip(&self, id: ClipId) -> Option<&Clip> {
        self.tracks.get(&id.track).and_then(|t| t.clip(id))
    }

    pub fn clip_mut(&mut self, id: ClipId) -> Option<&mut Clip> {
        self.tracks.get_mut(&id.track).and_then(|t| t.clip_mut(id))
    }

    pub fn clip_count(&self) -> usize {
        self.tracks.values().map(|t| t.clip_count()).sum()
    }

    pub fn move_clip(&mut self, id: ClipId, new_start: Time) -> Result<Time, TimelineError> {
        let track = self
            .tracks
            .get_mut(&id.track)
            .ok_or(TimelineError::TrackNotFound { id: id.track })?;
        track.move_clip(id, new_start)
    }

    // --- Keyframe management ---

    pub fn add_keyframe(
        &mut self,
        clip_id: ClipId,
        time: Time,
        value: KeyframeValue,
    ) -> Result<KeyframeId, TimelineError> {
        let kf_id = KeyframeId::from_raw(self.next_keyframe_id);
        self.next_keyframe_id += 1;

        let clip = self
            .tracks
            .get_mut(&clip_id.track)
            .and_then(|t| t.clip_mut(clip_id))
            .ok_or(TimelineError::ClipNotFound { id: clip_id })?;

        clip.add_keyframe(Keyframe::new(kf_id, time, value));
        Ok(kf_id)
    }

    pub fn remove_keyframe(
        &mut self,
        clip_id: ClipId,
        kf_id: KeyframeId,
    ) -> Result<Keyframe, TimelineError> {
        let clip = self
            .tracks
            .get_mut(&clip_id.track)
            .and_then(|t| t.clip_mut(clip_id))
            .ok_or(TimelineError::ClipNotFound { id: clip_id })?;

        clip.remove_keyframe(kf_id)
    }

    // --- Timeline duration ---

    /// The end time of the last clip across all tracks, or `Time::ZERO`
    /// if the timeline is empty.
    pub fn end_time(&self) -> Time {
        self.tracks
            .values()
            .filter_map(|t| t.clips().last().map(|c| c.end().seconds()))
            .fold(0.0, f64::max)
            .into()
    }

    /// Remove all tracks and clips. Id counters are not reset.
    pub fn clear(&mut self) {
        self.tracks.clear();
        self.track_order.clear();
    }


    /// All clips active at `time` across all tracks.
    /// A clip is active at `time` if `clip.start <= time < clip.start + clip.duration`.
    pub fn clips_at(&self, time: Time) -> Vec<&Clip> {
        self.tracks
            .values()
            .flat_map(|t| t.clips().iter())
            .filter(|clip| {
                let start = clip.start.seconds();
                let end = start + clip.duration.seconds();
                time.seconds() >= start && time.seconds() < end
            })
            .collect()
    }



    /// Trim a clip's duration without moving its start.
    pub fn trim_clip(&mut self, id: ClipId, new_duration: Duration)
        -> Result<Duration, TimelineError>
    {
        let clip = self.tracks.get_mut(&id.track)
            .and_then(|t| t.clip_mut(id))
            .ok_or(TimelineError::ClipNotFound { id })?;
        let previous = clip.duration;
        clip.set_duration(new_duration);
        Ok(previous)
    }

    /// Split a clip at `at_time` into two clips. The left clip keeps the
    /// original id; the right gets a new id. Returns the new clip's id.
    pub fn split_clip(&mut self, id: ClipId, at_time: Time)
        -> Result<ClipId, TimelineError>
    {
        let clip = self.tracks.get(&id.track)
            .and_then(|t| t.clip(id))
            .ok_or(TimelineError::ClipNotFound { id })?
            .clone();

        let new_id = ClipId::from_raw(self.next_clip_id, id.track);
        self.next_clip_id += 1;

        let (left, right) = clip.split(new_id, at_time)?;

        let track = self.tracks.get_mut(&id.track)
            .ok_or(TimelineError::TrackNotFound { id: id.track })?;
        track.replace_clip(id, vec![left, right])?;

        Ok(new_id)
    }


    /// Set a keyframe's value. Returns the previous value.
    pub fn set_keyframe_value(
        &mut self,
        clip_id: ClipId,
        kf_id: KeyframeId,
        value: KeyframeValue,
    ) -> Result<KeyframeValue, TimelineError> {
        let clip = self.tracks.get_mut(&clip_id.track)
            .and_then(|t| t.clip_mut(clip_id))
            .ok_or(TimelineError::ClipNotFound { id: clip_id })?;
        let kf = clip.keyframes_mut()
            .iter_mut()
            .find(|k| k.id == kf_id)
            .ok_or(TimelineError::KeyframeNotFound { id: kf_id })?;
        Ok(kf.set_value(value))
    }

}


impl<'a> IntoIterator for &'a Timeline {
    type Item = &'a TrackId;
    type IntoIter = std::slice::Iter<'a, TrackId>;

    fn into_iter(self) -> Self::IntoIter {
        self.track_order.iter()
    }
}