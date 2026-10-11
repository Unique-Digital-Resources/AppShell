//! Keyframe model: an animated value at a specific time within a clip.

use super::timeline::Time;
use std::fmt;

/// Stable identifier for a keyframe within a clip.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct KeyframeId(u64);

impl KeyframeId {
    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub fn as_raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for KeyframeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "kf{}", self.0)
    }
}

/// How a keyframe's value is interpolated to the next keyframe.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interpolation {
    /// Hold the value until the next keyframe (no blending).
    Step,
    /// Linear interpolation between this keyframe and the next.
    Linear,
    /// Smooth ease-in/ease-out (cubic).
    Smooth,
}

impl Default for Interpolation {
    fn default() -> Self {
        Interpolation::Linear
    }
}

/// A value stored in a keyframe.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum KeyframeValue {
    Float(f64),
    Int(i64),
    Bool(bool),
    Text(String),
}

/// A keyframe: an animated value at a specific time within a clip.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Keyframe {
    pub id: KeyframeId,
    pub time: Time,
    pub value: KeyframeValue,
    pub interpolation: Interpolation,
}

impl Keyframe {
    pub fn new(id: KeyframeId, time: Time, value: KeyframeValue) -> Self {
        Self {
            id,
            time,
            value,
            interpolation: Interpolation::default(),
        }
    }

    pub fn with_interpolation(mut self, interpolation: Interpolation) -> Self {
        self.interpolation = interpolation;
        self
    }

    pub fn set_value(&mut self, value: KeyframeValue) -> KeyframeValue {
        std::mem::replace(&mut self.value, value)
    }
}