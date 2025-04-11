use crate::math::{Quaternion, Vector3};

#[derive(Clone, Copy)]
pub struct Transform3D {
    pub position: Vector3,
    pub scale: Vector3,
    pub rotation: Quaternion
}

impl Transform3D {
    pub fn default() -> Transform3D {
        Transform3D {
            position: Vector3::zero(),
            scale: Vector3::new(1.0, 1.0, 1.0),
            rotation: Quaternion::identity()
        }
    }

    pub fn with_position(self: Self, new_position: Vector3) -> Transform3D {
        let mut result = self;
        result.position = new_position;
        result
    }

    pub fn with_scale(self: Self, new_scale: Vector3) -> Transform3D {
        let mut result = self;
        result.scale = new_scale;
        result
    }

    pub fn with_rotation(self: Self, new_rotation: Quaternion) -> Transform3D {
        let mut result = self;
        result.rotation = new_rotation;
        result
    }
}