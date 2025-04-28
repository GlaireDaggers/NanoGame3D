use rune::{Any, ContextError, Module};
use serde::Deserialize;

use crate::math::{Vector3, Vector4};

#[derive(Default, Clone, Copy)]
#[derive(Any)]
pub struct Color32 {
    #[rune(get, set)]
    pub r: u8,
    #[rune(get, set)]
    pub g: u8,
    #[rune(get, set)]
    pub b: u8,
    #[rune(get, set)]
    pub a: u8
}

impl Color32 {
    #[rune::function(keep, path = Self::new)]
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

    pub fn register_script(module: &mut Module) -> Result<(), ContextError> {
        module.ty::<Self>()?;
        module.function_meta(Self::new__meta)?;

        Ok(())
    }
}

#[derive(Clone, Copy)]
#[derive(Any)]
pub struct Rectangle {
    #[rune(get, set)]
    pub x: i32,
    #[rune(get, set)]
    pub y: i32,
    #[rune(get, set)]
    pub w: i32,
    #[rune(get, set)]
    pub h: i32
}

impl Rectangle {
    #[rune::function(keep, path = Self::new)]
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Rectangle {
        Rectangle { x, y, w, h }
    }

    pub fn register_script(module: &mut Module) -> Result<(), ContextError> {
        module.ty::<Self>()?;
        module.function_meta(Self::new__meta)?;

        Ok(())
    }
}

#[derive(Default, Clone, Copy, Deserialize)]
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