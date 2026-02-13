use spacetimedb::SpacetimeType;

/// A quaternion representing 3D rotation (orientation) in a right-handed, Y-up coordinate system.
/// This serves as a Serializable struct for SpacetimeDB with nalgebra [From] impls.
///
/// Positive rotation is counter-clockwise when looking down the axis toward the origin.
///
/// ```text
/// Identity (w=1, x=0, y=0, z=0):
///   Aligns local Forward to World -Z
///   Aligns local Up to World +Y
/// ```
///
#[derive(SpacetimeType, Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    /// Vector part (imaginary i)
    pub x: f32,
    /// Vector part (imaginary j)
    pub y: f32,
    /// Vector part (imaginary k)
    pub z: f32,
    /// Scalar part (real) - Set to 1.0 for Identity
    pub w: f32,
}

impl Default for Quat {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Quat {
    /// The "No Rotation" quaternion.
    /// Aligns the entity with the global axes (Forward = -Z, Up = +Y).
    pub const IDENTITY: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Quat { x, y, z, w }
    }
}
