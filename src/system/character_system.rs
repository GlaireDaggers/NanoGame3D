use hecs::{CommandBuffer, World};
use lazy_static::lazy_static;

use crate::{bsp::{bspcommon::transform_aabb, bspfile::{BspFile, MASK_SOLID}}, component::{charactercontroller::{CharacterController, CharacterInputState, CharacterState}, collider::ColliderBounds, fpview::FPView, mapmodel::MapModel, playerinput::PlayerInput, transform3d::Transform3D}, math::{Matrix4x4, Quaternion, Vector3, Vector4}, misc::AABB, gamestate::{InputState, MapData, TimeData}};

const GROUND_SLOPE_ANGLE: f32 = 45.0;
const STEP_HEIGHT: f32 = 20.0;
const GRAVITY: f32 = 300.0;
const FRICTION: f32 = 0.2;
const MAX_ACCEL: f32 = 10.0;
const AIR_ACCEL: f32 = 1.0;

lazy_static! {
    static ref GROUND_SLOPE_COS_ANGLE: f32 = GROUND_SLOPE_ANGLE.to_radians().cos();
}
/// System which initializes characters
pub fn character_init(world: &mut World) {
    // initialize character state
    let mut cmd_buffer = CommandBuffer::new();
    for (eid, cc) in world.query_mut::<&CharacterController>().without::<&CharacterState>() {
        cmd_buffer.insert_one(eid, CharacterState::new(cc.main_height));
        cmd_buffer.insert_one(eid, CharacterInputState::default());
    }
    cmd_buffer.run_on(world);
}

/// System which rotates characters according to an attached FPView
pub fn character_rotation_update(world: &mut World) {
    for (_, (_, transform, fpview)) in world.query_mut::<(&CharacterController, &mut Transform3D, &FPView)>() {
        transform.rotation = Quaternion::from_euler(Vector3::new(0.0, 0.0, fpview.yaw.to_radians()));
    }
}

/// System which allows characters with a PlayerInput component to receive input
pub fn character_input_update(input: &InputState, world: &mut World) {
    for (_, (state, transform, _)) in world.query_mut::<(&mut CharacterInputState, &Transform3D, &PlayerInput)>() {
        let rot_matrix = Matrix4x4::rotation(transform.rotation);

        let fwd = rot_matrix * Vector4::new(0.0, 1.0, 0.0, 0.0);
        let right = rot_matrix * Vector4::new(1.0, 0.0, 0.0, 0.0);

        let fwd = Vector3::new(fwd.x, fwd.y, fwd.z);
        let right = Vector3::new(right.x, right.y, right.z);

        let input_velocity = (fwd * input.move_y)
            + (right * input.move_x);

        state.input_move_dir = input_velocity;
        state.input_crouch = input.crouch;
        state.input_jump = input.jump;
    }
}

/// System which applies input to characters
pub fn character_apply_input_update(time: &TimeData, map_data: &MapData, world: &mut World) {
    for (_, (state, cc, input, transform)) in world.query_mut::<(&mut CharacterState, &mut CharacterController, &CharacterInputState, &Transform3D)>() {
        if state.grounded {
            // apply friction
            state.velocity = state.velocity - (state.velocity * FRICTION);
        }

        let wish_dir = Vector3::new(input.input_move_dir.x, input.input_move_dir.y, 0.0);
        let accel = if state.grounded { MAX_ACCEL } else { AIR_ACCEL };
        
        if wish_dir.length_sq() > 0.1 {
            let wish_speed = cc.move_speed * wish_dir.length();
            let wish_dir = wish_dir.normalized();
            let current_speed = wish_dir.dot(state.velocity);
            let add_speed = (wish_speed - current_speed).clamp(0.0, accel * cc.move_speed * time.delta_time);
            
            state.velocity = state.velocity + (wish_dir * add_speed);
        }

        if state.crouched && !input.input_crouch {
            // make sure we have enough room to uncrouch before doing so
            let box_extents = Vector3::new(cc.radius, cc.radius, cc.main_height * 0.5);
            let box_offset = Vector3::unit_z() * (cc.main_height * 0.5);
            let box_pos = transform.position + box_offset;

            if !map_data.map.box_check(MASK_SOLID, box_pos, box_extents) {
                state.crouched = false;
            }
        }
        else {
            state.crouched = input.input_crouch;
        }

        if state.grounded && input.input_jump {
            state.grounded = false;
            state.velocity.z = cc.jump_force;
        }

        state.height = if state.crouched { cc.crouch_height } else { cc.main_height };
        cc.height_offset = state.height * 0.5;
    }
}

/// System which controls movement of characters
pub fn character_update(time: &TimeData, map_data: &MapData, world: &mut World) {
    // gather map models
    let mut mapmodel_iter = world.query::<(&MapModel, &Transform3D)>();
    let mapmodels = mapmodel_iter
        .iter()
        .collect::<Vec<_>>();

    // gather characters
    let mut character_iter = world.query::<(&CharacterController, &mut CharacterState, &mut Transform3D)>();
    let characters = character_iter
        .iter()
        .collect::<Vec<_>>();

    // gather colliders
    let mut collider_iter = world.query::<(&ColliderBounds, &Transform3D)>();
    let colliders = collider_iter
        .iter()
        .collect::<Vec<_>>();

    // gather list of collidable entity bounds
    let mut collider_bounds = Vec::with_capacity(characters.len());
    for (ent, (cc, cstate, transform)) in &characters {
        let center = transform.position + Vector3::new(0.0, 0.0, cc.height_offset);
        let extents = Vector3::new(cc.radius, cc.radius, cstate.height);

        collider_bounds.push((*ent, AABB::center_extents(center, extents)));
    }
    for (ent, (cbounds, transform)) in &colliders {
        let local2world = Matrix4x4::scale(transform.scale)
            * Matrix4x4::rotation(transform.rotation)
            * Matrix4x4::translation(transform.position);

        let bounds = transform_aabb(&cbounds.bounds, local2world);
        collider_bounds.push((*ent, bounds));
    }

    // update character physics
    for (self_ent, (cc, cstate, transform)) in characters {
        // trace function which also checks against each map model entity & against other characters
        let trace_fn = |mask: u32, start: &Vector3, end: &Vector3, box_extents: &Vector3| {
            let mut trace = map_data.map.boxtrace(0, mask, *start, *end, *box_extents);

            for (e, (mapmodel, transform)) in &mapmodels {
                // transform trace start + end into model's local space
                let inv_r = transform.rotation.inverted();
                let inv_scale = 1.0 / transform.scale;

                let world2local = Matrix4x4::translation(transform.position * -1.0)
                    * Matrix4x4::rotation(inv_r)
                    * Matrix4x4::scale(inv_scale);

                let local_start = world2local * Vector4::new(start.x, start.y, start.z, 1.0);
                let local_end = world2local * Vector4::new(end.x, end.y, end.z, 1.0);

                let local_start = Vector3::new(local_start.x, local_start.y, local_start.z);
                let local_end = Vector3::new(local_end.x, local_end.y, local_end.z);

                let tr = map_data.map.boxtrace(mapmodel.model_idx + 1, mask, local_start, local_end, *box_extents);

                if tr.fraction < trace.fraction {
                    // transform trace results back into world space

                    let local2world = Matrix4x4::scale(transform.scale)
                        * Matrix4x4::rotation(transform.rotation)
                        * Matrix4x4::translation(transform.position);

                    let trace_end = local2world * Vector4::new(tr.end_pos.x, tr.end_pos.y, tr.end_pos.z, 1.0);
                    let trace_normal = local2world * Vector4::new(tr.hit_normal.x, tr.hit_normal.y, tr.hit_normal.z, 0.0);

                    trace = tr;
                    trace.end_pos = Vector3::new(trace_end.x, trace_end.y, trace_end.z);
                    trace.hit_normal = Vector3::new(trace_normal.x, trace_normal.y, trace_normal.z).normalized();
                    trace.entity = Some(*e);
                }
            }

            for (e, other_bounds) in &collider_bounds {
                if self_ent == *e {
                    continue;
                }

                if BspFile::trace_aabb(other_bounds, start, end, Some(box_extents), &mut trace) {
                    trace.entity = Some(*e);
                }
            }

            return trace;
        };

        let box_extents = Vector3::new(cc.radius, cc.radius, cstate.height * 0.5);
        let box_offset = Vector3::unit_z() * cc.height_offset;
        
        let box_pos = transform.position + box_offset;

        // sweep character sideways
        let move_vec_xy = Vector3::new(cstate.velocity.x, cstate.velocity.y, 0.0);

        let (box_pos, move_vec_xy) = if cstate.grounded && move_vec_xy.length_sq() > f32::EPSILON {
            let original_pos = box_pos;
            let original_move_vec_xy = move_vec_xy;

            // while on the ground, sweep up by step height, sweep sideways, then sweep back down by step height.
            let (box_pos, _, _) = map_data.map.trace_move(&box_pos, &Vector3::new(0.0, 0.0, STEP_HEIGHT), 1.0, false, box_extents, trace_fn);
            let (box_pos, move_vec_xy, _) = map_data.map.trace_move(&box_pos, &move_vec_xy, time.delta_time, true, box_extents, trace_fn);
            let (box_pos, _, trace) = map_data.map.trace_move(&box_pos, &Vector3::new(0.0, 0.0, -STEP_HEIGHT), 1.0, false, box_extents, trace_fn);

            // if we leave the ground, see if the ground is still close enough to step down
            let (box_pos, move_vec_xy) = if trace.fraction == 1.0 {
                let (new_pos, _, trace) = map_data.map.trace_move(&box_pos, &Vector3::new(0.0, 0.0, -STEP_HEIGHT), 1.0, false, box_extents, trace_fn);

                if trace.fraction < 1.0 {
                    (new_pos, move_vec_xy)
                }
                else {
                    (box_pos, move_vec_xy)
                }
            }
            else {
                // if we stepped onto ground that's too steep, reset back to original pos and just do a normal sweep instead
                if trace.hit_normal.z < *GROUND_SLOPE_COS_ANGLE {
                    let (box_pos, move_vec_xy, _) = map_data.map.trace_move(&original_pos, &original_move_vec_xy, time.delta_time, true, box_extents, trace_fn);
                    (box_pos, move_vec_xy)
                }
                else {
                    (box_pos, move_vec_xy)
                }
            };

            (box_pos, Vector3::new(move_vec_xy.x, move_vec_xy.y, f32::min(move_vec_xy.z, 0.0)))
        }
        else {
            let (box_pos, move_vec_xy, _) = map_data.map.trace_move(&box_pos, &move_vec_xy, time.delta_time, true, box_extents, trace_fn);
            (box_pos, move_vec_xy)
        };

        // sweep character down
        let move_vec_z = Vector3::unit_z() * cstate.velocity.z;
        let (box_pos, mut move_vec_z, trace) = map_data.map.trace_move(&box_pos, &move_vec_z, time.delta_time, !cstate.grounded, box_extents, trace_fn);

        // if we hit something while moving down, & slope is within threshold, set character to grounded state
        if trace.all_solid {
            // stuck, don't accumulate velocity
            cstate.velocity.z = 0.0;
            continue;
        }
        else if cstate.velocity.z < 0.0 && trace.fraction < 1.0 {
            if trace.hit_normal.z >= *GROUND_SLOPE_COS_ANGLE {
                cstate.grounded = true;
            }
            else {
                cstate.grounded = false;
            }
        }
        else if cstate.velocity.z > 0.0 && trace.fraction < 1.0 {
            // clamp velocity if we hit our head
            move_vec_z.z = 0.0;
        }
        else {
            cstate.grounded = false;
        }

        // update transform & character state
        transform.position = box_pos - box_offset;

        let prev_velocity = cstate.velocity;
        cstate.velocity = move_vec_xy + move_vec_z;

        cstate.velocity.z = f32::min(cstate.velocity.z, prev_velocity.z);
        
        // apply gravity
        if !cstate.grounded {
            cstate.velocity.z -= GRAVITY * time.delta_time;
        }
        else {
            cstate.velocity.z = -1.0;
        }
    }
}