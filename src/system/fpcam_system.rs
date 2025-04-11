use hecs::World;

use crate::{component::{camera::FPCamera, fpview::FPView, transform3d::Transform3D}, math::{Quaternion, Vector3}};

/// System which allows an FPCam to follow an entity with an FPView attached
pub fn fpcam_update(world: &mut World) {
    let mut camera_iter = world.query::<(&FPCamera, &mut Transform3D)>();
    let cameras = camera_iter
        .iter()
        .collect::<Vec<_>>();

    for (_, (fpcam, cam_transform)) in cameras {
        let target_fpview = world.get::<&FPView>(fpcam.follow_entity).unwrap();
        let target_transform = world.get::<&Transform3D>(fpcam.follow_entity).unwrap();

        cam_transform.rotation = Quaternion::from_euler(Vector3::new(target_fpview.pitch.to_radians(), 0.0, target_fpview.yaw.to_radians()));
        cam_transform.position = target_transform.position + Vector3::new(0.0, 0.0, target_fpview.eye_offset);
    }
}