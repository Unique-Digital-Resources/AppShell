# Timeline

Headless temporal editing model. Provides tracks, clips, keyframes, and time-based operations without imposing any playback engine, media decoding, or UI.

---

## Overview

```mermaid
flowchart TD
    subgraph TLM[Timeline]
        TL[Timeline<br/>tracks + track_order + id counters]
        TR[Track<br/>id + clips (sorted)]
        CL[Clip<br/>id + start + duration + keyframes]
        KF[Keyframe<br/>id + time + value + interpolation]
    end
    TLM -->|operations| OPS[TimelineOperation]
    OPS -->|apply| OUT[TimelineOutcome]
```

---

## Key Types

| Type | Fields | Purpose |
|---|---|---|
| `Timeline` | `tracks`, `track_order`, `next_track_id`, `next_clip_id`, `next_keyframe_id` | Container owning tracks and id counters |
| `Track` | `id: TrackId`, `clips: Vec<Clip>` | Independent timeline lane with non-overlapping clips |
| `Clip` | `id: ClipId`, `track: TrackId`, `start: Time`, `duration: Duration`, `keyframes: Vec<Keyframe>` | Time-bounded segment on a track |
| `Keyframe` | `id: KeyframeId`, `time: Time`, `value: KeyframeValue`, `interpolation: Interpolation` | Animated value at a specific time |
| `Time` | `f64` (newtype) | Absolute time position in seconds |
| `Duration` | `f64` (newtype) | Relative time length in seconds |
| `Interpolation` | `Step`, `Linear`, `Smooth` | How to interpolate between keyframes |
| `KeyframeValue` | `Float(f64)`, `Int(i64)`, `Bool(bool)`, `Text(String)` | Value stored in a keyframe |
| `ClipId` | `raw: u64`, `track: TrackId` | Composite id carrying its track |
| `TrackId` | `u64` (opaque) | Stable track identifier |
| `KeyframeId` | `u64` (opaque) | Stable keyframe identifier |

`Time` and `Duration` are separate newtypes to prevent accidental mixing of "when" and "how long."

---

## Timeline Methods

### Track management

| Method | Returns | Description |
|---|---|---|
| `create_track()` | `TrackId` | Create a new empty track |
| `insert_track(track)` | `TrackId` | Restore a track (for undo) |
| `remove_track(id)` | `Result<Track, TimelineError>` | Remove track and all its clips |
| `track(id)` | `Option<&Track>` | Read access |
| `track_mut(id)` | `Option<&mut Track>` | Mutable access |
| `tracks()` | `impl Iterator<Item = &Track>` | Iterate in order |
| `track_ids()` | `&[TrackId]` | Ordered track ids |
| `track_count()` | `usize` | Number of tracks |
| `contains_track(id)` | `bool` | Existence check |
| `move_track(id, new_index)` | `Result<usize, TimelineError>` | Reorder track, returns previous index |
| `clear()` | `()` | Remove all tracks (id counters preserved) |

### Clip management

| Method | Returns | Description |
|---|---|---|
| `insert_clip(track_id, start, duration)` | `Result<ClipId, TimelineError>` | Insert clip (rejects overlap) |
| `reinsert_clip(clip)` | `Result<ClipId, TimelineError>` | Restore a clip (for undo) |
| `remove_clip(id)` | `Result<Clip, TimelineError>` | Remove clip from its track |
| `clip(id)` | `Option<&Clip>` | Read access |
| `clip_mut(id)` | `Option<&mut Clip>` | Mutable access |
| `clip_count()` | `usize` | Total clips across all tracks |
| `move_clip(id, new_start)` | `Result<Time, TimelineError>` | Move clip (rejects overlap), returns previous start |
| `trim_clip(id, new_duration)` | `Result<Duration, TimelineError>` | Change duration without moving start |
| `split_clip(id, at_time)` | `Result<ClipId, TimelineError>` | Split into two clips, returns new clip's id |

### Keyframe management

| Method | Returns | Description |
|---|---|---|
| `add_keyframe(clip_id, time, value)` | `Result<KeyframeId, TimelineError>` | Add keyframe (sorted by time) |
| `remove_keyframe(clip_id, kf_id)` | `Result<Keyframe, TimelineError>` | Remove keyframe |
| `set_keyframe_value(clip_id, kf_id, value)` | `Result<KeyframeValue, TimelineError>` | Set value, returns previous |

### Queries

| Method | Returns | Description |
|---|---|---|
| `end_time()` | `Time` | End of last clip across all tracks |
| `clips_at(time)` | `Vec<&Clip>` | All clips active at `time` (across all tracks) |

---

## Clip Methods

| Method | Returns | Description |
|---|---|---|
| `end()` | `Time` | `start + duration` |
| `contains(time)` | `bool` | Whether `time` is in `[start, end)` |
| `keyframes()` | `&[Keyframe]` | All keyframes (sorted by time) |
| `keyframes_mut()` | `&mut [Keyframe]` | Mutable keyframe access |
| `keyframes_in_range(start, end)` | `Vec<&Keyframe>` | Keyframes in `[start, end)` |
| `add_keyframe(kf)` | `()` | Insert keyframe (sorted by time) |
| `remove_keyframe(id)` | `Result<Keyframe, TimelineError>` | Remove keyframe |
| `overlaps(other)` | `bool` | Whether this clip's time interval overlaps another |
| `set_duration(duration)` | `()` | Change duration |
| `split(new_id, at_time)` | `Result<(Clip, Clip), TimelineError>` | Split into left + right clips |

---

## Overlap Constraints

Clips within the same track **must not overlap**. The timeline enforces this on insert and move:

```text
Track A: [Clip 1: 0-5s] [Clip 2: 5-10s] [Clip 3: 10-15s]
                                          ↑ Clip 4 cannot start at 8s — overlaps Clip 3
```

- `insert_clip` checks overlap against all existing clips on the track.
- `move_clip` temporarily removes the clip, checks overlap at the new position, re-inserts if safe, or restores the original position and returns `ClipOverlap` error.

Clips on **different tracks** are independent and may overlap freely.

---

## Clip Split

`split_clip(id, at_time)` divides a clip into two at the specified time:

```text
Before:  [Clip: 0-10s]
                  split at 4s
After:   [Left: 0-4s] [Right: 4-10s]
```

The left clip keeps the original `ClipId`; the right clip gets a new id. Keyframes are not moved — they stay on whichever clip contains their time.

---

## Track Ordering

Tracks have a defined order via `track_order: Vec<TrackId>`. `move_track(id, new_index)` reorders tracks. This is separate from clip ordering (which is by start time within each track).

```rust
let mut timeline = Timeline::new();
let a = timeline.create_track();
let b = timeline.create_track();
let c = timeline.create_track();

assert_eq!(timeline.track_ids(), &[a, b, c]);

timeline.move_track(c, 0).unwrap();
assert_eq!(timeline.track_ids(), &[c, a, b]);
```

---

## Keyframe Interpolation

```rust
let kf = Keyframe::new(id, Time::new(2.0), KeyframeValue::Float(1.0))
    .with_interpolation(Interpolation::Smooth);
```

| Mode | Behavior |
|---|---|
| `Step` | Hold the value until the next keyframe (no blending) |
| `Linear` | Linear interpolation between this keyframe and the next |
| `Smooth` | Cubic ease-in/ease-out |

The foundation stores the interpolation mode but does **not** perform interpolation. A Domain Engine or UI reads the mode and computes interpolated values as needed.

---

## Operations

| Operation | Outcome | Undo |
|---|---|---|
| `CreateTrack` | `TrackCreated { id }` | Remove the track |
| `RemoveTrack { id }` | `TrackRemoved { track }` | Re-insert the track |
| `InsertClip { track_id, start, duration }` | `ClipInserted { id }` | Remove the clip |
| `RemoveClip { id }` | `ClipRemoved { clip }` | Re-insert the clip |
| `MoveClip { id, new_start }` | `ClipMoved { id, previous_start }` | Move back to previous start |
| `AddKeyframe { clip_id, time, value }` | `KeyframeAdded { id, clip_id }` | Remove the keyframe |
| `RemoveKeyframe { clip_id, kf_id }` | `KeyframeRemoved { keyframe, clip_id }` | Re-add the keyframe |
| `TrimClip { id, new_duration }` | `ClipTrimmed { id, previous_duration }` | Restore previous duration |
| `SplitClip { id, at_time }` | `ClipSplit { original_id, new_id }` | Remove new clip, restore original duration |
| `ClearTimeline` | `TimelineCleared` | *(limited — no stored tracks to restore)* |
| `SetKeyframeValue { clip_id, kf_id, value }` | `KeyframeValueChanged { previous_value }` | Restore previous value |

---

## Clips At Time

Query all clips active at a given time across all tracks:

```rust
let active = timeline.clips_at(Time::new(7.0));
for clip in &active {
    println!("Active: {} - {}", clip.start, clip.end());
}
```

A clip is active at `time` if `clip.start <= time < clip.start + clip.duration`.

---

## What Timeline Does NOT Own

| Concern | Owner |
|---|---|
| Playback engine (clock, frame rate) | Domain Engine / UI |
| Media decoding (audio, video frames) | External |
| Cross-track constraints (e.g., video must align with audio) | Domain Engine |
| Sub-clips / nested timelines | Domain Engine |
| Markers / annotations | Domain Engine |
| Interpolation computation | Domain Engine (reads the mode, computes values) |
| Tempo / frame rate | Domain Engine (Time is just f64 seconds) |

---

## Next Steps

- [06-layers.md](06-layers.md) — Layers environment.
- [07-node-graph.md](07-node-graph.md) — NodeGraph (composes Canvas2D).
- [11-relations.md](11-relations.md) — Relations (can define temporal dependencies).
- [15-operations-and-undo.md](15-operations-and-undo.md) — Operations and undo/redo patterns.