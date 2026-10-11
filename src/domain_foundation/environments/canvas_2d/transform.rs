//! 2D spatial primitives: vectors and transforms.
//!
//! These types describe *spatial state only*. They contain no rendering,
//! colour, or layout information.

use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

/// A 2D vector.
///
/// No coordinate-system orientation is implied. `x` and `y` are plain
/// floating-point components whose meaning is defined by the caller.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Construct a vector with both components equal to `v`.
    pub const fn splat(v: f64) -> Self {
        Self { x: v, y: v }
    }

    /// Vector dot product.
    pub fn dot(self, other: Vec2) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// Euclidean length (magnitude).
    pub fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    /// Normalised (unit-length) vector. Returns zero if length is zero.
    pub fn normalize(self) -> Vec2 {
        let len = self.length();
        if len == 0.0 {
            Vec2::zero()
        } else {
            Vec2::new(self.x / len, self.y / len)
        }
    }

    /// Linear interpolation between this vector and `other` by factor `t`.
    /// `t = 0.0` returns `self`, `t = 1.0` returns `other`.
    pub fn lerp(self, other: Vec2, t: f64) -> Vec2 {
        Vec2::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }

    /// Rotate this vector by `angle` radians (counter-clockwise).
    pub fn rotate(self, angle: f64) -> Vec2 {
        let cos = angle.cos();
        let sin = angle.sin();
        Vec2::new(
            self.x * cos - self.y * sin,
            self.x * sin + self.y * cos,
        )
    }

    /// Signed angle from this vector to `other`, in radians.
    /// Returns 0 if either vector has zero length.
    pub fn angle_to(self, other: Vec2) -> f64 {
        let len_prod = self.length() * other.length();
        if len_prod == 0.0 {
            return 0.0;
        }
        let dot = self.dot(other) / len_prod;
        let dot = dot.clamp(-1.0, 1.0);
        let angle = dot.acos();
        // Sign: cross product (z-component)
        let cross = self.x * other.y - self.y * other.x;
        if cross < 0.0 {
            -angle
        } else {
            angle
        }
    }

}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<f64> for Vec2 {
    type Output = Self;
    fn mul(self, s: f64) -> Self {
        Self { x: self.x * s, y: self.y * s }
    }
}

impl MulAssign<f64> for Vec2 {
    fn mul_assign(&mut self, s: f64) {
        self.x *= s;
        self.y *= s;
    }
}

/// A 2D spatial transform: translation, rotation (radians) and scale.
///
/// Stored in decomposed form for clarity and to avoid imposing a matrix
/// convention. When applied to a point, the intended order is
/// scale → rotate → translate. The foundation does not perform matrix
/// composition in Phase 2; downstream code that needs a matrix can build
/// one from these components.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub translation: Vec2,
    /// Rotation angle in radians. Positive direction is caller-defined.
    pub rotation: f64,
    pub scale: Vec2,
}

impl Transform {
    /// Identity transform: no translation, no rotation, unit scale.
    pub const fn identity() -> Self {
        Self {
            translation: Vec2::zero(),
            rotation: 0.0,
            scale: Vec2::splat(1.0),
        }
    }

    pub const fn from_translation(translation: Vec2) -> Self {
        Self { translation, rotation: 0.0, scale: Vec2::splat(1.0) }
    }

    pub const fn from_rotation(rotation: f64) -> Self {
        Self { translation: Vec2::zero(), rotation, scale: Vec2::splat(1.0) }
    }

    pub const fn from_scale(scale: Vec2) -> Self {
        Self { translation: Vec2::zero(), rotation: 0.0, scale }
    }

    pub const fn with_translation(self, translation: Vec2) -> Self {
        Self { translation, ..self }
    }

    pub const fn with_rotation(self, rotation: f64) -> Self {
        Self { rotation, ..self }
    }

    pub const fn with_scale(self, scale: Vec2) -> Self {
        Self { scale, ..self }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}