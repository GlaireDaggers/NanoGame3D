use hecs::Entity;

use crate::math::Vector3;

pub struct Door {
    pub auto_open: bool,
    pub close_pos: Vector3,
    pub open_pos: Vector3,
    pub move_speed: f32,
}

pub struct DoorLink {
    pub links: Vec<Entity>
}

pub struct DoorOpener {
}