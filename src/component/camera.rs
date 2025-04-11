use hecs::Entity;

use crate::misc::Rectangle;

#[derive(Clone, Copy)]
pub struct Camera {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub viewport_rect: Option<Rectangle>
}

impl Camera {
    pub fn default() -> Camera {
        Camera {
            fov: 60.0,
            near: 10.0,
            far: 10000.0,
            viewport_rect: None
        }
    }
}

#[derive(Clone, Copy)]
pub struct FPCamera {
    pub follow_entity: Entity
}

impl FPCamera {
    pub fn new(follow_entity: Entity) ->  FPCamera {
        FPCamera {
            follow_entity
        }
    }
}