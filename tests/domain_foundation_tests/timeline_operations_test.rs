//! Operation tests for the Timeline environment.

use app_shell::domain_foundation::environments::timeline::{
    Duration, KeyframeValue, Time, Timeline, TimelineError, TimelineOperation,
    TimelineOutcome, TrackId,
};

fn create_track_with_clip(timeline: &mut Timeline) -> (TrackId, app_shell::domain_foundation::environments::timeline::ClipId) {
    let track_id = match TimelineOperation::CreateTrack.apply(timeline).unwrap() {
        TimelineOutcome::TrackCreated { id } => id,
        _ => unreachable!(),
    };
    let outcome = TimelineOperation::InsertClip {
        track_id,
        start: Time::new(0.0),
        duration: Duration::new(5.0),
    }
    .apply(timeline)
    .unwrap();
    let clip_id = match outcome {
        TimelineOutcome::ClipInserted { id } => id,
        _ => unreachable!(),
    };
    (track_id, clip_id)
}

#[test]
fn create_track_operation() {
    let mut timeline = Timeline::new();
    let outcome = TimelineOperation::CreateTrack
        .apply(&mut timeline)
        .unwrap();

    let id = match outcome {
        TimelineOutcome::TrackCreated { id } => id,
        other => panic!("expected TrackCreated, got {other:?}"),
    };
    assert!(timeline.contains_track(id));
    assert_eq!(timeline.track_count(), 1);
}

#[test]
fn remove_track_operation_returns_track() {
    let mut timeline = Timeline::new();
    let track_id = timeline.create_track();
    timeline
        .insert_clip(track_id, Time::new(0.0), Duration::new(5.0))
        .unwrap();

    let outcome = TimelineOperation::RemoveTrack { id: track_id }
        .apply(&mut timeline)
        .unwrap();

    match outcome {
        TimelineOutcome::TrackRemoved { track } => {
            assert_eq!(track.id, track_id);
            assert_eq!(track.clip_count(), 1);
        }
        other => panic!("expected TrackRemoved, got {other:?}"),
    }
    assert!(!timeline.contains_track(track_id));
}

#[test]
fn insert_clip_operation() {
    let mut timeline = Timeline::new();
    let track_id = timeline.create_track();

    let outcome = TimelineOperation::InsertClip {
        track_id,
        start: Time::new(2.0),
        duration: Duration::new(3.0),
    }
    .apply(&mut timeline)
    .unwrap();

    let clip_id = match outcome {
        TimelineOutcome::ClipInserted { id } => id,
        other => panic!("expected ClipInserted, got {other:?}"),
    };
    assert!(timeline.clip(clip_id).is_some());
    assert_eq!(timeline.clip(clip_id).unwrap().start, Time::new(2.0));
}

#[test]
fn remove_clip_operation_returns_clip() {
    let mut timeline = Timeline::new();
    let track_id = timeline.create_track();
    let clip_id = timeline
        .insert_clip(track_id, Time::new(0.0), Duration::new(5.0))
        .unwrap();

    let outcome = TimelineOperation::RemoveClip { id: clip_id }
        .apply(&mut timeline)
        .unwrap();

    match outcome {
        TimelineOutcome::ClipRemoved { clip } => {
            assert_eq!(clip.id, clip_id);
            assert_eq!(clip.start, Time::new(0.0));
        }
        other => panic!("expected ClipRemoved, got {other:?}"),
    }
    assert!(timeline.clip(clip_id).is_none());
}

#[test]
fn move_clip_operation_records_previous_start() {
    let mut timeline = Timeline::new();
    let track_id = timeline.create_track();
    let clip_id = timeline
        .insert_clip(track_id, Time::new(0.0), Duration::new(5.0))
        .unwrap();

    let outcome = TimelineOperation::MoveClip {
        id: clip_id,
        new_start: Time::new(10.0),
    }
    .apply(&mut timeline)
    .unwrap();

    match outcome {
        TimelineOutcome::ClipMoved { id, previous_start } => {
            assert_eq!(id, clip_id);
            assert_eq!(previous_start, Time::new(0.0));
        }
        other => panic!("expected ClipMoved, got {other:?}"),
    }
    assert_eq!(timeline.clip(clip_id).unwrap().start, Time::new(10.0));
}

#[test]
fn add_keyframe_operation() {
    let mut timeline = Timeline::new();
    let track_id = timeline.create_track();
    let clip_id = timeline
        .insert_clip(track_id, Time::new(0.0), Duration::new(10.0))
        .unwrap();

    let outcome = TimelineOperation::AddKeyframe {
        clip_id,
        time: Time::new(2.5),
        value: KeyframeValue::Float(0.5),
    }
    .apply(&mut timeline)
    .unwrap();

    let kf_id = match outcome {
        TimelineOutcome::KeyframeAdded { id, clip_id: cid } => {
            assert_eq!(cid, clip_id);
            id
        }
        other => panic!("expected KeyframeAdded, got {other:?}"),
    };

    let clip = timeline.clip(clip_id).unwrap();
    assert_eq!(clip.keyframes().len(), 1);
    assert_eq!(clip.keyframes()[0].id, kf_id);
}

#[test]
fn remove_keyframe_operation_returns_keyframe() {
    let mut timeline = Timeline::new();
    let track_id = timeline.create_track();
    let clip_id = timeline
        .insert_clip(track_id, Time::new(0.0), Duration::new(10.0))
        .unwrap();
    let kf_id = timeline
        .add_keyframe(clip_id, Time::new(5.0), KeyframeValue::Float(1.0))
        .unwrap();

    let outcome = TimelineOperation::RemoveKeyframe {
        clip_id,
        kf_id,
    }
    .apply(&mut timeline)
    .unwrap();

    match outcome {
        TimelineOutcome::KeyframeRemoved { keyframe, clip_id: cid } => {
            assert_eq!(keyframe.id, kf_id);
            assert_eq!(cid, clip_id);
        }
        other => panic!("expected KeyframeRemoved, got {other:?}"),
    }
    assert_eq!(timeline.clip(clip_id).unwrap().keyframes().len(), 0);
}

#[test]
fn operations_on_missing_track_fail() {
    let mut timeline = Timeline::new();
    let fake_track = TrackId::from_raw(999);

    let err = TimelineOperation::InsertClip {
        track_id: fake_track,
        start: Time::new(0.0),
        duration: Duration::new(5.0),
    }
    .apply(&mut timeline)
    .unwrap_err();
    assert!(matches!(err, TimelineError::TrackNotFound { .. }));

    let err = TimelineOperation::RemoveTrack { id: fake_track }
        .apply(&mut timeline)
        .unwrap_err();
    assert!(matches!(err, TimelineError::TrackNotFound { .. }));
}

#[test]
fn sequence_of_operations_composes() {
    let mut timeline = Timeline::new();

    // Create two tracks.
    let track_a = match TimelineOperation::CreateTrack.apply(&mut timeline).unwrap() {
        TimelineOutcome::TrackCreated { id } => id,
        _ => unreachable!(),
    };
    let track_b = match TimelineOperation::CreateTrack.apply(&mut timeline).unwrap() {
        TimelineOutcome::TrackCreated { id } => id,
        _ => unreachable!(),
    };

    let outcome = TimelineOperation::InsertClip {
        track_id: track_a,
        start: Time::new(0.0),
        duration: Duration::new(10.0),
    }
    .apply(&mut timeline)
    .unwrap();
    let clip_a = match outcome {
        TimelineOutcome::ClipInserted { id } => id,
        _ => unreachable!(),
    };
    let outcome = TimelineOperation::InsertClip {
        track_id: track_b,
        start: Time::new(0.0),
        duration: Duration::new(15.0),
    }
    .apply(&mut timeline)
    .unwrap();
    let clip_b = match outcome {
        TimelineOutcome::ClipInserted { id } => id,
        _ => unreachable!(),
    };

    // Move clip_a.
    TimelineOperation::MoveClip {
        id: clip_a,
        new_start: Time::new(5.0),
    }
    .apply(&mut timeline)
    .unwrap();

    // Add a keyframe to clip_b.
    TimelineOperation::AddKeyframe {
        clip_id: clip_b,
        time: Time::new(7.5),
        value: KeyframeValue::Float(0.8),
    }
    .apply(&mut timeline)
    .unwrap();

    // Verify final state.
    assert_eq!(timeline.track_count(), 2);
    assert_eq!(timeline.clip_count(), 2);
    assert_eq!(timeline.clip(clip_a).unwrap().start, Time::new(5.0));
    assert_eq!(timeline.clip(clip_b).unwrap().keyframes().len(), 1);
    assert_eq!(timeline.end_time(), Time::new(15.0));
}
#[test]
fn trim_clip_operation() {
    let mut timeline = Timeline::new();
    let track = timeline.create_track();
    let clip_id = timeline
        .insert_clip(track, Time::new(0.0), Duration::new(10.0))
        .unwrap();

    let outcome = TimelineOperation::TrimClip {
        id: clip_id,
        new_duration: Duration::new(5.0),
    }
    .apply(&mut timeline)
    .unwrap();

    match outcome {
        TimelineOutcome::ClipTrimmed { previous_duration, .. } => {
            assert_eq!(previous_duration, Duration::new(10.0));
        }
        other => panic!("expected ClipTrimmed, got {other:?}"),
    }
    assert_eq!(timeline.clip(clip_id).unwrap().duration, Duration::new(5.0));
}

#[test]
fn split_clip_operation() {
    let mut timeline = Timeline::new();
    let track = timeline.create_track();
    let clip_id = timeline
        .insert_clip(track, Time::new(0.0), Duration::new(10.0))
        .unwrap();

    let outcome = TimelineOperation::SplitClip {
        id: clip_id,
        at_time: Time::new(4.0),
    }
    .apply(&mut timeline)
    .unwrap();

    let new_id = match outcome {
        TimelineOutcome::ClipSplit { original_id, new_id } => {
            assert_eq!(original_id, clip_id);
            new_id
        }
        other => panic!("expected ClipSplit, got {other:?}"),
    };

    // Original clip is now [0, 4).
    assert_eq!(timeline.clip(clip_id).unwrap().duration, Duration::new(4.0));
    // New clip is [4, 10).
    assert_eq!(timeline.clip(new_id).unwrap().start, Time::new(4.0));
    assert_eq!(timeline.clip(new_id).unwrap().duration, Duration::new(6.0));
}

#[test]
fn clear_timeline_operation() {
    let mut timeline = Timeline::new();
    timeline.create_track();
    timeline.create_track();

    let outcome = TimelineOperation::ClearTimeline
        .apply(&mut timeline)
        .unwrap();
    assert!(matches!(outcome, TimelineOutcome::TimelineCleared));
    assert_eq!(timeline.track_count(), 0);
}

#[test]
fn set_keyframe_value_operation() {
    use app_shell::domain_foundation::environments::timeline::KeyframeValue;

    let mut timeline = Timeline::new();
    let track = timeline.create_track();
    let clip_id = timeline
        .insert_clip(track, Time::new(0.0), Duration::new(10.0))
        .unwrap();
    let kf_id = timeline
        .add_keyframe(clip_id, Time::new(5.0), KeyframeValue::Float(1.0))
        .unwrap();

    let outcome = TimelineOperation::SetKeyframeValue {
        clip_id,
        kf_id,
        value: KeyframeValue::Float(2.0),
    }
    .apply(&mut timeline)
    .unwrap();

    match outcome {
        TimelineOutcome::KeyframeValueChanged { previous_value, .. } => {
            assert_eq!(previous_value, KeyframeValue::Float(1.0));
        }
        other => panic!("expected KeyframeValueChanged, got {other:?}"),
    }
}