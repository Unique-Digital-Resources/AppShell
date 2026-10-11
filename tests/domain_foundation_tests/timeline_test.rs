//! Model and query tests for the Timeline environment.

use app_shell::domain_foundation::environments::timeline::{
    Duration, Time, Timeline, TimelineError,
};

#[test]
fn new_timeline_is_empty() {
    let timeline = Timeline::new();
    assert_eq!(timeline.track_count(), 0);
    assert_eq!(timeline.clip_count(), 0);
    assert_eq!(timeline.end_time(), Time::ZERO);
}

#[test]
fn create_track_returns_unique_ids() {
    let mut timeline = Timeline::new();
    let a = timeline.create_track();
    let b = timeline.create_track();
    let c = timeline.create_track();
    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(a, c);
    assert_eq!(timeline.track_count(), 3);
}

#[test]
fn insert_clip_places_clip_on_track() {
    let mut timeline = Timeline::new();
    let track_id = timeline.create_track();

    let clip_id = timeline
        .insert_clip(track_id, Time::new(1.0), Duration::new(5.0))
        .unwrap();

    let clip = timeline.clip(clip_id).unwrap();
    assert_eq!(clip.start, Time::new(1.0));
    assert_eq!(clip.duration, Duration::new(5.0));
    assert_eq!(clip.end(), Time::new(6.0));
    assert_eq!(timeline.clip_count(), 1);
}

#[test]
fn clips_within_track_must_not_overlap() {
    let mut timeline = Timeline::new();
    let track_id = timeline.create_track();

    // First clip: [0, 5)
    timeline
        .insert_clip(track_id, Time::new(0.0), Duration::new(5.0))
        .unwrap();

    // Overlapping clip: [3, 8) — should fail
    let err = timeline
        .insert_clip(track_id, Time::new(3.0), Duration::new(5.0))
        .unwrap_err();
    assert!(matches!(err, TimelineError::ClipOverlap { .. }));

    // Non-overlapping clip: [5, 10) — should succeed
    timeline
        .insert_clip(track_id, Time::new(5.0), Duration::new(5.0))
        .unwrap();

    assert_eq!(timeline.clip_count(), 2);
}

#[test]
fn clips_on_different_tracks_are_independent() {
    let mut timeline = Timeline::new();
    let track_a = timeline.create_track();
    let track_b = timeline.create_track();

    // Both clips at [0, 5) but on different tracks — should succeed.
    timeline
        .insert_clip(track_a, Time::new(0.0), Duration::new(5.0))
        .unwrap();
    timeline
        .insert_clip(track_b, Time::new(0.0), Duration::new(5.0))
        .unwrap();

    assert_eq!(timeline.clip_count(), 2);
}

#[test]
fn remove_clip_removes_from_track() {
    let mut timeline = Timeline::new();
    let track_id = timeline.create_track();
    let clip_id = timeline
        .insert_clip(track_id, Time::new(0.0), Duration::new(5.0))
        .unwrap();

    let removed = timeline.remove_clip(clip_id).unwrap();
    assert_eq!(removed.id, clip_id);
    assert!(timeline.clip(clip_id).is_none());
    assert_eq!(timeline.clip_count(), 0);
}

#[test]
fn remove_track_removes_all_clips() {
    let mut timeline = Timeline::new();
    let track_id = timeline.create_track();
    timeline
        .insert_clip(track_id, Time::new(0.0), Duration::new(5.0))
        .unwrap();
    timeline
        .insert_clip(track_id, Time::new(5.0), Duration::new(5.0))
        .unwrap();

    assert_eq!(timeline.clip_count(), 2);

    let removed = timeline.remove_track(track_id).unwrap();
    assert_eq!(removed.clip_count(), 2);
    assert_eq!(timeline.track_count(), 0);
    assert_eq!(timeline.clip_count(), 0);
}

#[test]
fn end_time_is_max_clip_end_across_tracks() {
    let mut timeline = Timeline::new();
    let track_a = timeline.create_track();
    let track_b = timeline.create_track();

    timeline
        .insert_clip(track_a, Time::new(0.0), Duration::new(10.0))
        .unwrap();
    timeline
        .insert_clip(track_b, Time::new(0.0), Duration::new(15.0))
        .unwrap();

    assert_eq!(timeline.end_time(), Time::new(15.0));
}

#[test]
fn keyframe_management() {
    let mut timeline = Timeline::new();
    let track_id = timeline.create_track();
    let clip_id = timeline
        .insert_clip(track_id, Time::new(0.0), Duration::new(10.0))
        .unwrap();

    let kf_id = timeline
        .add_keyframe(clip_id, Time::new(2.0), app_shell::domain_foundation::environments::timeline::KeyframeValue::Float(1.0))
        .unwrap();

    let clip = timeline.clip(clip_id).unwrap();
    assert_eq!(clip.keyframes().len(), 1);
    assert_eq!(clip.keyframes()[0].id, kf_id);

    let removed = timeline.remove_keyframe(clip_id, kf_id).unwrap();
    assert_eq!(removed.id, kf_id);
    assert_eq!(timeline.clip(clip_id).unwrap().keyframes().len(), 0);
}

#[test]
fn move_clip_changes_start_time() {
    let mut timeline = Timeline::new();
    let track_id = timeline.create_track();
    let clip_id = timeline
        .insert_clip(track_id, Time::new(0.0), Duration::new(5.0))
        .unwrap();

    let previous = timeline.move_clip(clip_id, Time::new(10.0)).unwrap();
    assert_eq!(previous, Time::new(0.0));
    assert_eq!(timeline.clip(clip_id).unwrap().start, Time::new(10.0));
}

#[test]
fn move_clip_rejects_overlap() {
    let mut timeline = Timeline::new();
    let track_id = timeline.create_track();

    let clip_a = timeline
        .insert_clip(track_id, Time::new(0.0), Duration::new(5.0))
        .unwrap();
    let _clip_b = timeline
        .insert_clip(track_id, Time::new(5.0), Duration::new(5.0))
        .unwrap();

    // Moving clip_a to [3, 8) would overlap clip_b [5, 10).
    let err = timeline.move_clip(clip_a, Time::new(3.0)).unwrap_err();
    assert!(matches!(err, TimelineError::ClipOverlap { .. }));

    // clip_a should still be at its original position.
    assert_eq!(timeline.clip(clip_a).unwrap().start, Time::new(0.0));
}

#[test]
fn clips_are_returned_sorted_by_start_time() {
    let mut timeline = Timeline::new();
    let track_id = timeline.create_track();

    timeline
        .insert_clip(track_id, Time::new(10.0), Duration::new(5.0))
        .unwrap();
    timeline
        .insert_clip(track_id, Time::new(0.0), Duration::new(5.0))
        .unwrap();
    timeline
        .insert_clip(track_id, Time::new(5.0), Duration::new(5.0))
        .unwrap();

    let track = timeline.track(track_id).unwrap();
    let starts: Vec<_> = track.clips().iter().map(|c| c.start.seconds()).collect();
    assert_eq!(starts, vec![0.0, 5.0, 10.0]);
}

#[test]
fn insert_clip_on_missing_track_fails() {
    let mut timeline = Timeline::new();
    let err = timeline
        .insert_clip(
            app_shell::domain_foundation::environments::timeline::TrackId::from_raw(999),
            Time::new(0.0),
            Duration::new(5.0),
        )
        .unwrap_err();
    assert!(matches!(err, TimelineError::TrackNotFound { .. }));
}

#[test]
fn keyframe_interpolation_modes() {
    use app_shell::domain_foundation::environments::timeline::{
        Interpolation, Keyframe, KeyframeId, KeyframeValue, Time,
    };

    let kf = Keyframe::new(KeyframeId::from_raw(0), Time::new(1.0), KeyframeValue::Float(0.5));
    assert_eq!(kf.interpolation, Interpolation::Linear);

    let kf_step = Keyframe::new(KeyframeId::from_raw(1), Time::new(2.0), KeyframeValue::Float(1.0))
        .with_interpolation(Interpolation::Step);
    assert_eq!(kf_step.interpolation, Interpolation::Step);
}

#[test]
fn timeline_clear_empties() {
    let mut timeline = Timeline::new();
    let track = timeline.create_track();
    timeline.insert_clip(track, Time::new(0.0), Duration::new(5.0)).unwrap();
    assert_eq!(timeline.track_count(), 1);
    
    timeline.clear();
    assert_eq!(timeline.track_count(), 0);
    assert_eq!(timeline.clip_count(), 0);
}

#[test]
fn timeline_partial_eq_works() {
    let a = Timeline::new();
    let b = Timeline::new();
    assert_eq!(a, b);
}

#[test]
fn clips_at_finds_active_clips() {
    let mut timeline = Timeline::new();
    let track = timeline.create_track();

    // Clip [0, 5)
    timeline.insert_clip(track, Time::new(0.0), Duration::new(5.0)).unwrap();
    // Clip [10, 15)
    timeline.insert_clip(track, Time::new(10.0), Duration::new(5.0)).unwrap();

    // At time 2.0 — first clip is active.
    let active = timeline.clips_at(Time::new(2.0));
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].start, Time::new(0.0));

    // At time 12.0 — second clip is active.
    let active = timeline.clips_at(Time::new(12.0));
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].start, Time::new(10.0));

    // At time 7.0 — gap between clips, nothing active.
    let active = timeline.clips_at(Time::new(7.0));
    assert!(active.is_empty());

    // At time 0.0 — start of first clip (inclusive).
    let active = timeline.clips_at(Time::new(0.0));
    assert_eq!(active.len(), 1);

    // At time 5.0 — end of first clip (exclusive).
    let active = timeline.clips_at(Time::new(5.0));
    assert!(active.is_empty());
}

#[test]
fn clips_at_across_multiple_tracks() {
    let mut timeline = Timeline::new();
    let track_a = timeline.create_track();
    let track_b = timeline.create_track();

    timeline.insert_clip(track_a, Time::new(0.0), Duration::new(10.0)).unwrap();
    timeline.insert_clip(track_b, Time::new(5.0), Duration::new(10.0)).unwrap();

    // At time 7.0 — both clips are active.
    let active = timeline.clips_at(Time::new(7.0));
    assert_eq!(active.len(), 2);
}

#[test]
fn clip_contains_time() {
    let mut timeline = Timeline::new();
    let track = timeline.create_track();
    let clip_id = timeline
        .insert_clip(track, Time::new(5.0), Duration::new(10.0))
        .unwrap();

    let clip = timeline.clip(clip_id).unwrap();

    // Within range.
    assert!(clip.contains(Time::new(5.0)));   // start (inclusive)
    assert!(clip.contains(Time::new(10.0)));  // middle
    assert!(clip.contains(Time::new(14.9))); // near end

    // Outside range.
    assert!(!clip.contains(Time::new(4.9)));  // before start
    assert!(!clip.contains(Time::new(15.0))); // end (exclusive)
}

#[test]
fn keyframes_in_range() {
    use app_shell::domain_foundation::environments::timeline::{
        KeyframeId, KeyframeValue,
    };

    let mut timeline = Timeline::new();
    let track = timeline.create_track();
    let clip_id = timeline
        .insert_clip(track, Time::new(0.0), Duration::new(20.0))
        .unwrap();

    timeline
        .add_keyframe(clip_id, Time::new(2.0), KeyframeValue::Float(1.0))
        .unwrap();
    timeline
        .add_keyframe(clip_id, Time::new(5.0), KeyframeValue::Float(2.0))
        .unwrap();
    timeline
        .add_keyframe(clip_id, Time::new(10.0), KeyframeValue::Float(3.0))
        .unwrap();
    timeline
        .add_keyframe(clip_id, Time::new(15.0), KeyframeValue::Float(4.0))
        .unwrap();

    let clip = timeline.clip(clip_id).unwrap();

    // Range [5, 15) — includes kf at 5 and 10, excludes 15.
    let kfs = clip.keyframes_in_range(Time::new(5.0), Time::new(15.0));
    assert_eq!(kfs.len(), 2);
    assert_eq!(kfs[0].time, Time::new(5.0));
    assert_eq!(kfs[1].time, Time::new(10.0));

    // Range [0, 20) — all keyframes.
    let kfs = clip.keyframes_in_range(Time::new(0.0), Time::new(20.0));
    assert_eq!(kfs.len(), 4);

    // Range [12, 14) — no keyframes.
    let kfs = clip.keyframes_in_range(Time::new(12.0), Time::new(14.0));
    assert!(kfs.is_empty());
}

#[test]
fn move_track_changes_order() {
    let mut timeline = Timeline::new();
    let a = timeline.create_track();
    let b = timeline.create_track();
    let c = timeline.create_track();

    assert_eq!(timeline.track_ids(), &[a, b, c]);

    let prev = timeline.move_track(c, 0).unwrap();
    assert_eq!(prev, 2);
    assert_eq!(timeline.track_ids(), &[c, a, b]);

    // Move a to end.
    let prev = timeline.move_track(a, 2).unwrap();
    assert_eq!(prev, 1);
    assert_eq!(timeline.track_ids(), &[c, b, a]);
}

#[test]
fn move_track_out_of_range_fails() {
    let mut timeline = Timeline::new();
    let a = timeline.create_track();
    let _b = timeline.create_track();

    let err = timeline.move_track(a, 5).unwrap_err();
    assert!(matches!(
        err,
        app_shell::domain_foundation::environments::timeline::TimelineError::InvalidTrackIndex { .. }
    ));
}

#[test]
fn move_track_on_missing_track_fails() {
    let mut timeline = Timeline::new();
    timeline.create_track();

    let fake = app_shell::domain_foundation::environments::timeline::TrackId::from_raw(999);
    let err = timeline.move_track(fake, 0).unwrap_err();
    assert!(matches!(
        err,
        app_shell::domain_foundation::environments::timeline::TimelineError::TrackNotFound { .. }
    ));
}

#[test]
fn track_ids_preserves_creation_order() {
    let mut timeline = Timeline::new();
    let a = timeline.create_track();
    let b = timeline.create_track();
    let c = timeline.create_track();

    assert_eq!(timeline.track_ids(), &[a, b, c]);
    assert_eq!(timeline.track_count(), 3);
}

#[test]
fn timeline_into_iterator() {
    let mut timeline = Timeline::new();
    let a = timeline.create_track();
    let b = timeline.create_track();

    let ids: Vec<_> = (&timeline).into_iter().copied().collect();
    assert_eq!(ids, vec![a, b]);
}