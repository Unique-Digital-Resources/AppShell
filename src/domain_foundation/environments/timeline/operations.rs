//! Timeline operations.

use super::clip::ClipId;
use super::keyframe::{KeyframeId, KeyframeValue};
use super::timeline::{Duration, Time, Timeline};
use super::track::TrackId;
use super::error::TimelineError;

#[derive(Debug, Clone, PartialEq)]
pub enum TimelineOperation {
    CreateTrack,
    RemoveTrack { id: TrackId },
    InsertClip { track_id: TrackId, start: Time, duration: Duration },
    RemoveClip { id: ClipId },
    MoveClip { id: ClipId, new_start: Time },
    AddKeyframe { clip_id: ClipId, time: Time, value: KeyframeValue },
    RemoveKeyframe { clip_id: ClipId, kf_id: KeyframeId },
    /// Change a clip's duration without moving its start.
    TrimClip { id: ClipId, new_duration: Duration },
    /// Split a clip at `at_time` into two clips.
    SplitClip { id: ClipId, at_time: Time },
    /// Remove all tracks and clips.
    ClearTimeline,
    /// Set a keyframe's value.
    SetKeyframeValue { clip_id: ClipId, kf_id: KeyframeId, value: KeyframeValue },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimelineOutcome {
    TrackCreated { id: TrackId },
    TrackRemoved { track: super::track::Track },
    ClipInserted { id: ClipId },
    ClipRemoved { clip: super::clip::Clip },
    ClipMoved { id: ClipId, previous_start: Time },
    KeyframeAdded { id: KeyframeId, clip_id: ClipId },
    KeyframeRemoved { keyframe: super::keyframe::Keyframe, clip_id: ClipId },
    ClipTrimmed { id: ClipId, previous_duration: Duration },
    ClipSplit { original_id: ClipId, new_id: ClipId },
    TimelineCleared,
    KeyframeValueChanged { clip_id: ClipId, kf_id: KeyframeId, previous_value: KeyframeValue },
}

impl TimelineOperation {
    pub fn apply(self, timeline: &mut Timeline) -> Result<TimelineOutcome, TimelineError> {
        match self {
            TimelineOperation::CreateTrack => {
                let id = timeline.create_track();
                Ok(TimelineOutcome::TrackCreated { id })
            }
            TimelineOperation::RemoveTrack { id } => {
                let track = timeline.remove_track(id)?;
                Ok(TimelineOutcome::TrackRemoved { track })
            }
            TimelineOperation::InsertClip { track_id, start, duration } => {
                let id = timeline.insert_clip(track_id, start, duration)?;
                Ok(TimelineOutcome::ClipInserted { id })
            }
            TimelineOperation::RemoveClip { id } => {
                let clip = timeline.remove_clip(id)?;
                Ok(TimelineOutcome::ClipRemoved { clip })
            }
            TimelineOperation::MoveClip { id, new_start } => {
                let previous_start = timeline.move_clip(id, new_start)?;
                Ok(TimelineOutcome::ClipMoved { id, previous_start })
            }
            TimelineOperation::AddKeyframe { clip_id, time, value } => {
                let id = timeline.add_keyframe(clip_id, time, value)?;
                Ok(TimelineOutcome::KeyframeAdded { id, clip_id })
            }
            TimelineOperation::RemoveKeyframe { clip_id, kf_id } => {
                let keyframe = timeline.remove_keyframe(clip_id, kf_id)?;
                Ok(TimelineOutcome::KeyframeRemoved { keyframe, clip_id })
            }
            TimelineOperation::TrimClip { id, new_duration } => {
                let previous_duration = timeline.trim_clip(id, new_duration)?;
                Ok(TimelineOutcome::ClipTrimmed { id, previous_duration })
            }
            TimelineOperation::SplitClip { id, at_time } => {
                let new_id = timeline.split_clip(id, at_time)?;
                Ok(TimelineOutcome::ClipSplit { original_id: id, new_id })
            }
            TimelineOperation::ClearTimeline => {
                timeline.clear();
                Ok(TimelineOutcome::TimelineCleared)
            }
            TimelineOperation::SetKeyframeValue { clip_id, kf_id, value } => {
                let previous_value = timeline.set_keyframe_value(clip_id, kf_id, value)?;
                Ok(TimelineOutcome::KeyframeValueChanged { clip_id, kf_id, previous_value })
            }
        }
    }
}