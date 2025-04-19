use std::sync::Arc;

use crate::graphics::{buffer::Buffer, model::{MeshVertex, Model}};

pub struct RenderMesh {
    pub mesh: Arc<Model>
}

pub struct SkinnedMesh {
    // temporary vertex storage
    pub vtx_array: Vec<MeshVertex>,
    // one vertex buffer per mesh part per mesh group (index as vtx_buffer[mesh_group][mesh_part])
    pub vtx_buffer: Vec<Vec<Buffer>>,
}

impl RenderMesh {
    pub fn new(mesh: Arc<Model>) -> RenderMesh {
        RenderMesh { mesh }
    }
}

impl SkinnedMesh {
    pub fn new(mesh: &Arc<Model>) -> SkinnedMesh {
        SkinnedMesh {
            vtx_array: Vec::new(),
            vtx_buffer: mesh.meshes.iter().map(|x| {
                x.parts.iter().map(|x| {
                    Buffer::new((x.vertices.len() * size_of::<MeshVertex>()) as isize)
                }).collect()
            }).collect()
        }
    }
}