use gamemath::{Vec2, Vec3, Vec4};

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