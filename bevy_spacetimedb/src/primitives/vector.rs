use spacetimedb::SpacetimeType;

/// A 3-dimensional vector in a right-handed, Y-up coordinate system. This serves
/// as a Serializable struct for SpacetimeDB with nalgebra [From] impls for math work.
///
/// +X is "right", -X is "left"
/// +Y is "up", -Y is "down"
/// +Z is "backward", -Z is "forward"
///
/// ```text
///      Y (Up)
///      |
///      |   -Z (Forward / Into Screen)
///      |  /
///      | /
///      o --------- X (Right)
///     /
///    /
///   Z (Backward / Out of Screen)
/// ```
#[derive(SpacetimeType, Debug, Default, Clone, Copy, PartialEq)]
pub struct Vec3 {
    /// +X is "right", -X is "left"
    pub x: f32,
    /// +Y is "up", -Y is "down"
    pub y: f32,
    /// +Z is "backward", -Z is "forward"
    pub z: f32,
}

impl Vec3 {
    // Basic Constants
    pub const ZERO: Vec3 = Vec3::new(0.0, 0.0, 0.0);
    pub const ONE: Vec3 = Vec3::new(1.0, 1.0, 1.0);

    // Y-axis constants
    pub const UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);
    pub const DOWN: Vec3 = Vec3::new(0.0, -1.0, 0.0);

    // X-axis constants
    pub const RIGHT: Vec3 = Vec3::new(1.0, 0.0, 0.0);
    pub const LEFT: Vec3 = Vec3::new(-1.0, 0.0, 0.0);

    // Z-axis constants
    pub const BACKWARD: Vec3 = Vec3::new(0.0, 0.0, 1.0);
    pub const FORWARD: Vec3 = Vec3::new(0.0, 0.0, -1.0);

    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    /// Returns a 2D vector using the X and Z axes (ignoring height/Y).
    /// Commonly used for pathfinding and ground-based movement.
    #[inline(always)]
    pub const fn xz(&self) -> Vec2 {
        Vec2::new(self.x, self.z)
    }
}

/// A 2-dimensional vector, representing the horizontal plane (X, Z) of the 3d world.
/// This serves as a Serializable struct for SpacetimeDB with nalgebra [From] impls for math work.
/// This is intentionally kept as x/z instead of x/y to avoid confusion with what the values are representing,
/// however nalgebra uses xy since it doesn't assume a coordinate system or dimensionality.
#[derive(SpacetimeType, Debug, Default, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub z: f32,
}
impl Vec2 {
    pub const ZERO: Vec2 = Vec2::new(0.0, 0.0);
    pub const ONE: Vec2 = Vec2::new(1.0, 1.0);

    #[inline(always)]
    pub const fn new(x: f32, z: f32) -> Self {
        Vec2 { x, z }
    }

    /// Converts this Vec2 into a Vec3 by providing a Y (height, up/down) value.
    #[inline(always)]
    pub const fn extend(&self, y: f32) -> Vec3 {
        Vec3::new(self.x, y, self.z)
    }
}
