use hecs::World;

use crate::{component::{basicanim::{AnimationLoopMode, BasicAnim}, meshpose::MeshPose, rendermesh::RenderMesh}, TimeData};

// system which samples basic animations & writes results into ModelPose
pub fn basic_animation_system(time: &TimeData, world: &mut World) {
    let mut anim_iter = world.query::<(&RenderMesh, &mut BasicAnim, &mut MeshPose)>();

    for (_, (mesh, anim, pose)) in anim_iter.iter() {
        pose.sample(&mesh.mesh, anim.animation_id, anim.animation_time);

        let a = &mesh.mesh.animations[anim.animation_id];
        anim.animation_time = match anim.loop_mode {
            AnimationLoopMode::Wrap => {
                (anim.animation_time + time.delta_time) % a.duration
            },
            AnimationLoopMode::Clamp => {
                (anim.animation_time + time.delta_time).min(a.duration)
            }
        };
    }
}