use crate::math::Vector3;

#[derive(Clone, Copy)]
pub struct CharacterController {
    pub radius: f32,
    pub height_offset: f32,
    pub move_speed: f32,
    pub jump_force: f32,
    pub main_height: f32,
    pub crouch_height: f32,
}

#[derive(Clone, Copy)]
pub struct CharacterState {
    pub height: f32,
    pub velocity: Vector3,
    pub grounded: bool,
    pub crouched: bool,
}

#[derive(Clone, Copy)]
pub struct CharacterInputState {
    pub input_move_dir: Vector3,
    pub input_crouch: bool,
    pub input_jump: bool,
}

impl CharacterController {
    pub fn default() -> CharacterController {
        CharacterController {
            radius: 16.0,
            main_height: 48.0,
            crouch_height: 16.0,
            height_offset: 24.0,
            move_speed: 200.0,
            jump_force: 150.0,
        }
    }
}

impl CharacterState {
    pub fn new(height: f32) -> CharacterState {
        CharacterState {
            height: height,
            velocity: Vector3::zero(),
            grounded: false,
            crouched: false,
        }
    }
}

impl CharacterInputState {
    pub fn default() -> CharacterInputState {
        CharacterInputState {
            input_move_dir: Vector3::zero(),
            input_crouch: false,
            input_jump: false,
        }
    }
}