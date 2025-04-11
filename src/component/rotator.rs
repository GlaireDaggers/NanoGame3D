use crate::math::Vector3;

#[derive(Clone, Copy)]
pub struct Rotator {
    pub rot_axis: Vector3,
    pub rot_speed: f32,
}