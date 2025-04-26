use crate::{asset_loader::ModelHandle, graphics::{buffer::Buffer, model::MeshVertex}};

pub struct RenderMesh {
    pub mesh: ModelHandle
}

pub struct SkinnedMesh {
    // temporary vertex storage
    pub vtx_array: Vec<MeshVertex>,
    // one vertex buffer per mesh part per mesh group (index as vtx_buffer[mesh_group][mesh_part])
    pub vtx_buffer: Vec<Vec<Buffer>>,
}

impl RenderMesh {
    pub fn new(mesh: ModelHandle) -> RenderMesh {
        RenderMesh { mesh }
    }
}

impl SkinnedMesh {
    pub fn new(mesh: &ModelHandle) -> SkinnedMesh {
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