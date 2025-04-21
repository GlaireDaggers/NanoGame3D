use serde::Deserialize;

use crate::{graphics::{anim::{Color32Curve, Vector2Curve}, material::Material}, math::Vector3, serialization::SerializedResource};

#[derive(Deserialize)]
pub enum EffectEmissionShape {
    Point { origin: Vector3 },
    Box { origin: Vector3, extents: Vector3 },
    Sphere { origin: Vector3, inner_radius: f32, outer_radius: f32 },
    Ring { origin: Vector3, axis: Vector3, inner_radius: f32, outer_radius: f32 }
}

#[derive(Deserialize)]
pub struct EffectSpritesheet {
    pub rows: u32,
    pub columns: u32,
    pub random_start: bool,
    pub timescale: f32,
}

#[derive(Deserialize)]
pub enum EffectDisplay {
    None,
    Sprite {
        material: SerializedResource<Material>,
        sheet: Option<EffectSpritesheet>,
        size: Vector2Curve,
        color: Color32Curve,
    },
}

#[derive(Deserialize)]
pub struct EffectEmission {
    pub max_particles: u32,
    pub max_bursts: Option<u32>,
    pub particles_per_burst: u32,
    pub burst_interval: f32,
    pub world_space: bool,
    pub shape: EffectEmissionShape,
}

#[derive(Deserialize)]
pub struct EffectInit {
    pub lifetime_min: f32,
    pub lifetime_max: f32,
    pub angle_min: f32,
    pub angle_max: f32,
    pub angle_axis: Vector3,
    pub angle_axis_spread: f32,
    pub direction: Vector3,
    pub direction_spread: f32,
    pub velocity_min: f32,
    pub velocity_max: f32,
    pub angular_velocity_min: f32,
    pub angular_velocity_max: f32,
}

#[derive(Deserialize)]
pub struct EffectAcceleration {
    pub gravity: Vector3,
    pub linear_damp: f32,
    pub angular_damp: f32,
    pub radial_accel: f32,
    pub orbit_accel: f32,
    pub orbit_axis: Vector3,
}

#[derive(Deserialize)]
pub struct EffectEmitter {
    pub emit: EffectEmission,
    pub init: EffectInit,
    pub accel: EffectAcceleration,
    pub display: EffectDisplay,
}

#[derive(Deserialize)]
pub struct EffectData {
    pub emitters: Vec<EffectEmitter>
}