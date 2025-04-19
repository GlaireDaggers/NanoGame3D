use hecs::World;

use crate::{component::{basicanim::{AnimationLoopMode, BasicAnim, BasicLerpAnim}, meshpose::{MeshPose, PoseBlendMode}, rendermesh::RenderMesh}, TimeData};

/// system which samples basic animations & writes results into ModelPose's local pose
pub fn basic_animation_system(time: &TimeData, world: &mut World) {
    let mut anim_iter = world.query::<(&RenderMesh, &mut BasicAnim, &mut MeshPose)>();

    for (_, (mesh, anim, pose)) in anim_iter.iter() {
        pose.sample(&mesh.mesh, None, anim.animation_id, anim.animation_time, PoseBlendMode::Replace, 0.0);

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

    let mut anim_iter = world.query::<(&RenderMesh, &mut BasicLerpAnim, &mut MeshPose)>();

    for (_, (mesh, anim, pose)) in anim_iter.iter() {
        pose.sample(&mesh.mesh, None, anim.animation1_id, anim.animation1_time, PoseBlendMode::Replace, 0.0);
        pose.sample(&mesh.mesh, None, anim.animation2_id, anim.animation2_time, PoseBlendMode::Mix, anim.mix);

        let a1 = &mesh.mesh.animations[anim.animation1_id];
        anim.animation1_time = match anim.loop_mode {
            AnimationLoopMode::Wrap => {
                (anim.animation1_time + time.delta_time) % a1.duration
            },
            AnimationLoopMode::Clamp => {
                (anim.animation1_time + time.delta_time).min(a1.duration)
            }
        };

        let a2 = &mesh.mesh.animations[anim.animation2_id];
        anim.animation2_time = match anim.loop_mode {
            AnimationLoopMode::Wrap => {
                (anim.animation2_time + time.delta_time) % a2.duration
            },
            AnimationLoopMode::Clamp => {
                (anim.animation2_time + time.delta_time).min(a2.duration)
            }
        };
    }
}

/// system which computes transforms for each ModelPose from stored local pose
pub fn compute_pose_transforms(world: &mut World) {
    let mut pose_iter = world.query::<(&RenderMesh, &mut MeshPose)>();

    for (_, (mesh, pose)) in pose_iter.iter() {
        pose.compute_pose_transforms(&mesh.mesh);
    }
}