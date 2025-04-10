use gamemath::{Mat4, Vec2, Vec3, Vec4};

pub const VEC2_ZERO: Vector2    = Vector2 { x: 0.0, y: 0.0 };
pub const VEC2_ONE: Vector2     = Vector2 { x: 1.0, y: 1.0 };
pub const VEC2_UNIT_X: Vector2  = Vector2 { x: 1.0, y: 0.0 };
pub const VEC2_UNIT_Y: Vector2  = Vector2 { x: 0.0, y: 1.0 };

pub const VEC3_ZERO: Vector3    = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
pub const VEC3_ONE: Vector3     = Vector3 { x: 1.0, y: 1.0, z: 1.0 };
pub const VEC3_UNIT_X: Vector3  = Vector3 { x: 1.0, y: 0.0, z: 0.0 };
pub const VEC3_UNIT_Y: Vector3  = Vector3 { x: 0.0, y: 1.0, z: 0.0 };
pub const VEC3_UNIT_Z: Vector3  = Vector3 { x: 0.0, y: 0.0, z: 1.0 };

pub const VEC4_ZERO: Vector4    = Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
pub const VEC4_ONE: Vector4     = Vector4 { x: 1.0, y: 1.0, z: 1.0, w: 1.0 };
pub const VEC4_UNIT_X: Vector4  = Vector4 { x: 1.0, y: 0.0, z: 0.0, w: 0.0 };
pub const VEC4_UNIT_Y: Vector4  = Vector4 { x: 0.0, y: 1.0, z: 0.0, w: 0.0 };
pub const VEC4_UNIT_Z: Vector4  = Vector4 { x: 0.0, y: 0.0, z: 1.0, w: 0.0 };
pub const VEC4_UNIT_W: Vector4  = Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

#[derive(Clone, Copy)]
pub struct Color32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}

impl Color32 {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Color32 {
        Color32 { r, g, b, a }
    }
}

pub type Vector2 = Vec2<f32>;
pub type Vector3 = Vec3<f32>;
pub type Vector4 = Vec4<f32>;

pub fn vec2_div(lhs: Vector2, rhs: Vector2) -> Vector2 {
    Vector2::new(lhs.x / rhs.x, lhs.y / rhs.y)
}

pub fn vec2_mul(lhs: Vector2, rhs: Vector2) -> Vector2 {
    Vector2::new(lhs.x * rhs.x, lhs.y * rhs.y)
}

pub fn mat4_translation(translation: Vector3) -> Mat4 {
    // Mat4::translated is busted
    Mat4 { rows: [
        Vector4::new(1.0, 0.0, 0.0, translation.x),
        Vector4::new(0.0, 1.0, 0.0, translation.y),
        Vector4::new(0.0, 0.0, 1.0, translation.z),
        Vector4::new(0.0, 0.0, 0.0, 1.0),
    ]}
}