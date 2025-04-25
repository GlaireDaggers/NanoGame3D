use hecs::World;

use crate::{bsp::bspfile::BspFile, component::{flycam::FlyCam, fpview::FPView, playerinput::PlayerInput, transform3d::Transform3D}, math::{Matrix4x4, Quaternion, Vector3, Vector4}, gamestate::{InputState, TimeData}};

/// System which allows player to control a FlyCam
pub fn flycam_system_update(input: &InputState, time: &TimeData, map: &BspFile, world: &mut World) {
    let collider_bounds = Vector3::new(15.0, 15.0, 15.0);

    for (_, (transform, fpview, _, _)) in world.query_mut::<(&mut Transform3D, &FPView, &PlayerInput, &FlyCam)>() {
        transform.rotation = Quaternion::from_euler(Vector3::new(fpview.pitch.to_radians(), 0.0, fpview.yaw.to_radians()));
        let rot_matrix = Matrix4x4::rotation(transform.rotation);

        let camera_fwd = rot_matrix * Vector4::new(0.0, -1.0, 0.0, 0.0);
        let camera_right = rot_matrix * Vector4::new(1.0, 0.0, 0.0, 0.0);

        let camera_fwd = Vector3::new(camera_fwd.x, camera_fwd.y, camera_fwd.z);
        let camera_right = Vector3::new(camera_right.x, camera_right.y, camera_right.z);

        let camera_velocity = (camera_fwd * 100.0 * input.move_y)
            + (camera_right * 100.0 * input.move_x);

        let (new_pos, _, _) = map.trace_move(&transform.position, &camera_velocity, time.delta_time, true, collider_bounds,
            |mask, start, end, box_extents| {
                return map.boxtrace(0, mask, *start, *end, *box_extents);
            });
        transform.position = new_pos;
    }
}