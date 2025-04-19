use std::sync::Arc;

use crate::{graphics::model::{Model, ModelAnimationClip}, math::Matrix4x4};

pub struct MeshPose {
    // array of one object-space transform per node in hierarchy
    pub pose : Vec<Matrix4x4>
}

impl MeshPose {
    pub fn init(mesh: &Arc<Model>) -> MeshPose {
        MeshPose { pose: vec![Matrix4x4::identity();mesh.nodes.len()] }
    }

    fn sample_node(self: &mut Self, model: &Arc<Model>, parent_xform: Matrix4x4, node_idx: &mut usize, anim: &ModelAnimationClip, time: f32) {
        let node = &model.nodes[*node_idx];

        let node_xform = if let Some(ch) = anim.channels.get(node_idx) {
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

            Matrix4x4::scale(scale) *
            Matrix4x4::rotation(rotation) *
            Matrix4x4::translation(position)
        }
        else {
            node.transform
        } * parent_xform;

        self.pose[*node_idx] = node_xform;
        *node_idx += 1;

        for _ in 0..node.num_children {
            self.sample_node(model, node_xform, node_idx, anim, time);
        }
    }

    pub fn sample(self: &mut Self, model: &Arc<Model>, anim_id: usize, time: f32) {
        let anim = &model.animations[anim_id];

        let mut node_idx = 0;
        while node_idx < model.nodes.len() {
            self.sample_node(model, Matrix4x4::identity(), &mut node_idx, anim, time);
        }
    }
}