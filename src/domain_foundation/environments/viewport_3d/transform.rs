//! 3D spatial primitives: vectors and transforms.

use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign};

/// A 3D vector.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }

    pub const fn splat(v: f64) -> Self {
        Self { x: v, y: v, z: v }
    }

    /// Vector dot product.
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Vector cross product.
    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Euclidean length.
    pub fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    /// Normalised (unit-length) vector. Returns zero if length is zero.
    pub fn normalised(self) -> Self {
        let len = self.length();
        if len == 0.0 {
            Self::zero()
        } else {
            Self::new(self.x / len, self.y / len, self.z / len)
        }
    }

    /// Linear interpolation between this vector and `other` by factor `t`.
    /// `t = 0.0` returns `self`, `t = 1.0` returns `other`.
    pub fn lerp(self, other: Vec3, t: f64) -> Vec3 {
        Vec3::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
            self.z + (other.z - self.z) * t,
        )
    }

    /// Euclidean distance to `other`.
    pub fn distance_to(self, other: Vec3) -> f64 {
        (other - self).length()
    }

}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, s: f64) -> Self {
        Self { x: self.x * s, y: self.y * s, z: self.z * s }
    }
}

impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, s: f64) {
        self.x *= s;
        self.y *= s;
        self.z *= s;
    }
}

/// A 3D spatial transform: translation, rotation (Euler angles) and scale.
///
/// Rotation is stored as a [`Vec3`] of pitch (x), yaw (y), and roll (z)
/// in radians. The foundation does not perform matrix composition; Domain
/// Engines that need a transformation matrix can build one from these
/// components.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform3D {
    pub translation: Vec3,
    /// Euler angles in radians: (pitch, yaw, roll).
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Transform3D {
    /// Identity transform: no translation, no rotation, unit scale.
    pub const fn identity() -> Self {
        Self {
            translation: Vec3::zero(),
            rotation: Vec3::zero(),
            scale: Vec3::splat(1.0),
        }
    }

    pub const fn from_translation(translation: Vec3) -> Self {
        Self { translation, rotation: Vec3::zero(), scale: Vec3::splat(1.0) }
    }

    pub const fn from_rotation(rotation: Vec3) -> Self {
        Self { translation: Vec3::zero(), rotation, scale: Vec3::splat(1.0) }
    }

    pub const fn from_scale(scale: Vec3) -> Self {
        Self { translation: Vec3::zero(), rotation: Vec3::zero(), scale }
    }

    pub const fn with_translation(self, translation: Vec3) -> Self {
        Self { translation, ..self }
    }

    pub const fn with_rotation(self, rotation: Vec3) -> Self {
        Self { rotation, ..self }
    }

    pub const fn with_scale(self, scale: Vec3) -> Self {
        Self { scale, ..self }
    }
}

impl Default for Transform3D {
    fn default() -> Self {
        Self::identity()
    }
}