use std::sync::Arc;

use crate::{graphics::model::{Model, ModelAnimationClip}, math::{Matrix4x4, Quaternion, Vector3}};

use super::transform3d::Transform3D;

#[derive(Clone, Copy)]
pub enum PoseBlendMode {
    Replace,
    Mix,
    Add
}

pub struct MeshPose {
    // array of local space transforms (position, rotation, scale) per node in hierarchy
    pub local_pose: Vec<Transform3D>,
    // array of one object-space transform per node in hierarchy
    pub pose : Vec<Matrix4x4>,
}

impl MeshPose {
    pub fn init(mesh: &Arc<Model>) -> MeshPose {
        MeshPose {
            local_pose: vec![Transform3D::default();mesh.nodes.len()],
            pose: vec![Matrix4x4::identity();mesh.nodes.len()],
        }
    }

    fn sample_pose(model: &Arc<Model>, anim: &ModelAnimationClip, time: f32, node_idx: usize) -> (Vector3, Quaternion, Vector3) {
        let node = &model.nodes[node_idx];

        if let Some(ch) = anim.channels.get(&node_idx) {
            let position = if let Some(curve) = &ch.translation {
                curve.sample(time)
            }
            else {
                node.rest_pos
            };

            let rotation = if let Some(curve) = &ch.rotation {
                curve.sample(time)
            }
            else {
                node.rest_rot
            };

            let scale = if let Some(curve) = &ch.scale {
                curve.sample(time)
            }
            else {
                node.rest_scale
            };

            (position, rotation, scale)
        }
        else {
            (node.rest_pos, node.rest_rot, node.rest_scale)
        }
    }

    fn sample_node_local(self: &mut Self, model: &Arc<Model>, node_idx: usize, anim: &ModelAnimationClip, time: f32, blend_mode: PoseBlendMode, amount: f32) {
        let node_xform = Self::sample_pose(model, anim, time, node_idx);

        match blend_mode {
            PoseBlendMode::Replace => {
                self.local_pose[node_idx] = Transform3D::default()
                    .with_position(node_xform.0)
                    .with_rotation(node_xform.1)
                    .with_scale(node_xform.2);
            }
            PoseBlendMode::Mix => {
                let mut stored_pose = self.local_pose[node_idx];
                stored_pose.position = Vector3::lerp(stored_pose.position, node_xform.0, amount);
                stored_pose.rotation = Quaternion::slerp(stored_pose.rotation, node_xform.1, amount);
                stored_pose.scale = Vector3::lerp(stored_pose.scale, node_xform.2, amount);

                self.local_pose[node_idx] = stored_pose;
            }
            PoseBlendMode::Add => {
                // note: for additive poses, we actually need to also sample the first frame of the animation,
                // and calculate the current frame's offset relative to that
                // this way, the first frame is effectively treated as a "reference pose",
                // and the offset from that reference is what gets additively applied to the current pose

                let ref_pose = Self::sample_pose(model, anim, 0.0, node_idx);
                let pos_offset = node_xform.0 - ref_pose.0;
                let rot_offset = node_xform.1 * ref_pose.1.inverted();
                let scale_offset = node_xform.2 / ref_pose.2;

                // note: first frame needs a non-zero scale, otherwise we can't compute a relative scale value
                if ref_pose.2.x.abs() <= f32::EPSILON || 
                    ref_pose.2.y.abs() <= f32::EPSILON ||
                    ref_pose.2.z.abs() <= f32::EPSILON {
                    println!("INVALID REF SCALE IN ADDITIVE ANIMATION!");
                }

                let mut stored_pose = self.local_pose[node_idx];
                stored_pose.position = stored_pose.position + (pos_offset * amount);
                stored_pose.rotation = stored_pose.rotation * Quaternion::slerp(Quaternion::identity(), rot_offset, amount);
                stored_pose.scale = stored_pose.scale * Vector3::lerp(Vector3::new(1.0, 1.0, 1.0), scale_offset, amount);

                self.local_pose[node_idx] = stored_pose;
            }
        }
    }

    fn sample_node_local_recursive(self: &mut Self, model: &Arc<Model>, node_idx: &mut usize, anim: &ModelAnimationClip, time: f32, blend_mode: PoseBlendMode, amount: f32) {
        self.sample_node_local(model, *node_idx, anim, time, blend_mode, amount);

        let node = &model.nodes[*node_idx];
        for _ in 0..node.num_children {
            *node_idx += 1;
            self.sample_node_local_recursive(model, node_idx, anim, time, blend_mode, amount);
        }
    }

    fn compute_node_transform(self: &mut Self, model: &Arc<Model>, parent_xform: Matrix4x4, node_idx: &mut usize) {
        let node = &model.nodes[*node_idx];

        let node_xform = self.local_pose[*node_idx];
        let local_to_world = Matrix4x4::scale(node_xform.scale)
            * Matrix4x4::rotation(node_xform.rotation)
            * Matrix4x4::translation(node_xform.position)
            * parent_xform;

        self.pose[*node_idx] = local_to_world;
        *node_idx += 1;

        for _ in 0..node.num_children {
            self.compute_node_transform(model, local_to_world, node_idx);
        }
    }

    pub fn sample(self: &mut Self, model: &Arc<Model>, root_node: Option<usize>, anim_id: usize, time: f32, blend_mode: PoseBlendMode, amount: f32) {
        let anim = &model.animations[anim_id];

        if let Some(root) = root_node {
            let mut node_idx = root;
            self.sample_node_local_recursive(model, &mut node_idx, anim, time, blend_mode, amount);
        }
        else {
            for node_idx in 0..model.nodes.len() {
                self.sample_node_local(model, node_idx, anim, time, blend_mode, amount);
            }
        }
    }

    /// Compute transform matrices for each node based on currently stored local pose
    pub fn compute_pose_transforms(self: &mut Self, model: &Arc<Model>) {
        let mut node_idx = 0;
        while node_idx < model.nodes.len() {
            self.compute_node_transform(model, Matrix4x4::identity(), &mut node_idx);
        }
    }
}