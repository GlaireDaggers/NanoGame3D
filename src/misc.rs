use crate::math::{Vector3, Vector4};

#[derive(Default, Clone, Copy)]
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

    pub fn from_vec4(vec: Vector4) -> Color32 {
        let r = (vec.x.clamp(0.0, 1.0) * 255.0) as u8;
        let g = (vec.y.clamp(0.0, 1.0) * 255.0) as u8;
        let b = (vec.z.clamp(0.0, 1.0) * 255.0) as u8;
        let a = (vec.w.clamp(0.0, 1.0) * 255.0) as u8;

        Color32::new(r, g, b, a)
    }

    pub fn to_vec4(self: Self) -> Vector4 {
        Vector4::new(self.r as f32 / 255.0, self.g as f32 / 255.0, self.b as f32 / 255.0, self.a as f32 / 255.0)
    }
}

#[derive(Clone, Copy)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32
}

impl Rectangle {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Rectangle {
        Rectangle { x, y, w, h }
    }
}

#[derive(Default, Clone, Copy)]
pub struct AABB {
    pub center: Vector3,
    pub extents: Vector3,
}

impl AABB {
    pub fn center_extents(center: Vector3, extents: Vector3) -> AABB {
        AABB { center, extents }
    }

    pub fn min_max(min: Vector3, max: Vector3) -> AABB {
        AABB { center: (min + max) * 0.5, extents: (max - min) * 0.5 }
    }

    pub fn inflate(self: Self, amount: Vector3) -> AABB {
        AABB { center: self.center, extents: self.extents + amount }
    }
    
    pub fn with_extents(self: Self, extents: Vector3) -> AABB {
        AABB { center: self.center, extents: extents }
    }

    pub fn min(self: &Self) -> Vector3 {
        self.center - self.extents
    }

    pub fn max(self: &Self) -> Vector3 {
        self.center + self.extents
    }
}