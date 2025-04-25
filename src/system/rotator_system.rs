use hecs::World;

use crate::{component::{rotator::Rotator, transform3d::Transform3D}, gamestate::TimeData, math::Quaternion};

/// System which rotates entities with a Rotator component
pub fn rotator_system_update(time: &TimeData, world: &mut World) {
    for (_, (transform, rotator)) in world.query_mut::<(&mut Transform3D, &Rotator)>() {
        let a = (rotator.rot_speed * time.delta_time) * 0.5;
        let sa = a.sin();
        let ca = a.cos();
        let rot = Quaternion::new(rotator.rot_axis.x * sa, rotator.rot_axis.y * sa, rotator.rot_axis.z * sa, ca);

        transform.rotation = transform.rotation * rot;
    }
}