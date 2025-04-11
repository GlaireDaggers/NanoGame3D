use hecs::World;

use crate::{component::{charactercontroller::{CharacterController, CharacterState}, fpview::FPView, playerinput::PlayerInput}, InputState, TimeData};

const LOOK_SPEED: f32 = 90.0;
const CROUCH_SPEED: f32 = 120.0;

/// System which allows player to control yaw/pitch of FPView
pub fn fpview_input_system_update(input: &InputState, time: &TimeData, world: &mut World) {
    for (_, (fpview, _)) in world.query_mut::<(&mut FPView, &PlayerInput)>() {
        if input.look_x.abs() >= 0.1 {
            fpview.yaw -= ((input.look_x - 0.1) * 1.1111) * LOOK_SPEED * time.delta_time;
        }

        if input.look_y.abs() >= 0.1 {
            fpview.pitch += ((input.look_y - 0.1) * 1.1111) * LOOK_SPEED * time.delta_time;
        }

        if fpview.yaw < 0.0 {
            fpview.yaw += 360.0;
        }
        else if fpview.yaw > 360.0 {
            fpview.yaw -= 360.0;
        }

        fpview.pitch = fpview.pitch.clamp(-90.0, 90.0);
    }
}

/// System which updates eye offset of FPView
pub fn fpview_eye_update(time: &TimeData, world: &mut World) {
    for (_, (fpview, cc, cstate)) in world.query_mut::<(&mut FPView, &CharacterController, &CharacterState)>() {
        let cur_height = fpview.eye_offset;
        let target_height = if cstate.crouched {
            cc.crouch_height - 5.0
        }
        else {
            cc.main_height - 5.0
        };

        let height_delta = target_height - cur_height;
        let height_delta = height_delta.abs().clamp(0.0, CROUCH_SPEED * time.delta_time) * height_delta.signum();

        fpview.eye_offset = cur_height + height_delta;
    }
}