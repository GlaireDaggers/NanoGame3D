use std::{mem::offset_of, sync::Arc};

use gltf::{buffer::Data, Document, Mesh, Node, Primitive};

use crate::{asset_loader::load_material, math::{Matrix4x4, Vector2, Vector4}, misc::Color32};

use super::{buffer::Buffer, material::Material, shader::Shader};

pub struct MeshVertex {
    pub position: Vector4,
    pub normal: Vector4,
    pub tangent: Vector4,
    pub texcoord0: Vector2,
    pub texcoord1: Vector2,
    pub color: Color32,
    pub joints: [u8;2],
    pub weights: [u8;2],
}

impl MeshVertex {
    pub fn setup_vtx_arrays(shader: &Shader) {
        let position = shader.get_attribute_location("in_position");
        let normal = shader.get_attribute_location("in_normal");
        let tangent = shader.get_attribute_location("in_tangent");
        let texcoord0 = shader.get_attribute_location("in_texcoord0");
        let texcoord1 = shader.get_attribute_location("in_texcoord1");
        let color = shader.get_attribute_location("in_color");
        let joints = shader.get_attribute_location("in_joints");
        let weights = shader.get_attribute_location("in_weights");

        unsafe {
            gl::EnableVertexAttribArray(position);
            gl::EnableVertexAttribArray(normal);
            gl::EnableVertexAttribArray(tangent);
            gl::EnableVertexAttribArray(texcoord0);
            gl::EnableVertexAttribArray(texcoord1);
            gl::EnableVertexAttribArray(color);
            gl::EnableVertexAttribArray(joints);
            gl::EnableVertexAttribArray(weights);

            gl::VertexAttribPointer(position, 4, gl::FLOAT, gl::FALSE, size_of::<MeshVertex>() as i32, offset_of!(MeshVertex, position) as *const _);
            gl::VertexAttribPointer(normal, 4, gl::FLOAT, gl::FALSE, size_of::<MeshVertex>() as i32, offset_of!(MeshVertex, normal) as *const _);
            gl::VertexAttribPointer(tangent, 4, gl::FLOAT, gl::FALSE, size_of::<MeshVertex>() as i32, offset_of!(MeshVertex, tangent) as *const _);
            gl::VertexAttribPointer(texcoord0, 2, gl::FLOAT, gl::FALSE, size_of::<MeshVertex>() as i32, offset_of!(MeshVertex, texcoord0) as *const _);
            gl::VertexAttribPointer(texcoord1, 2, gl::FLOAT, gl::FALSE, size_of::<MeshVertex>() as i32, offset_of!(MeshVertex, texcoord1) as *const _);
            gl::VertexAttribPointer(color, 4, gl::UNSIGNED_BYTE, gl::TRUE, size_of::<MeshVertex>() as i32, offset_of!(MeshVertex, color) as *const _);
            gl::VertexAttribPointer(joints, 2, gl::UNSIGNED_BYTE, gl::FALSE, size_of::<MeshVertex>() as i32, offset_of!(MeshVertex, joints) as *const _);
            gl::VertexAttribPointer(weights, 2, gl::UNSIGNED_BYTE, gl::TRUE, size_of::<MeshVertex>() as i32, offset_of!(MeshVertex, weights) as *const _);
        }
    }
}

pub struct MeshPart {
    pub material_index: usize,
    pub winding: gl::types::GLenum,
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u16>,
    pub topology: gl::types::GLenum,
    pub buffers: Option<(Buffer, Buffer)>,
}

impl MeshPart {
    pub fn new(topology: gl::types::GLenum) -> MeshPart {
        MeshPart {
            material_index: 0,
            winding: gl::CCW,
            vertices: Vec::new(),
            indices: Vec::new(),
            topology,
            buffers: None,
        }
    }

    pub fn from_gltf(primitive: Primitive, buffers: &[Data]) -> MeshPart {
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        let positions = reader.read_positions().unwrap();
        let mut vertices = Vec::new();

        // GLTF -> NG3D: (.x, -.z, .y)

        for pos in positions {
            let vtx = MeshVertex {
                position: Vector4::new(pos[0], -pos[2], pos[1], 1.0),
                normal: Vector4::zero(),
                tangent: Vector4::zero(),
                texcoord0: Vector2::zero(),
                texcoord1: Vector2::zero(),
                color: Color32::new(255, 255, 255, 255),
                joints: [0, 0],
                weights: [0, 0],
            };

            vertices.push(vtx);
        }

        if let Some(normals) = reader.read_normals() {
            for (idx, val) in normals.enumerate() {
                vertices[idx].normal = Vector4::new(val[0], -val[2], val[1], 0.0);
            }
        }

        if let Some(tangents) = reader.read_tangents() {
            for (idx, val) in tangents.enumerate() {
                vertices[idx].tangent = Vector4::new(val[0], -val[2], val[1], 0.0);
            }
        }

        if let Some(texcoords) = reader.read_tex_coords(0) {
            for (idx, val) in texcoords.into_f32().enumerate() {
                vertices[idx].texcoord0 = Vector2::new(val[0], val[1]);
            }
        }

        if let Some(texcoords) = reader.read_tex_coords(1) {
            for (idx, val) in texcoords.into_f32().enumerate() {
                vertices[idx].texcoord1 = Vector2::new(val[0], val[1]);
            }
        }

        if let Some(colors) = reader.read_colors(0) {
            for (idx, val) in colors.into_rgba_u8().enumerate() {
                vertices[idx].color = Color32::new(val[0], val[1], val[2], val[3]);
            }
        }

        if let Some(joints) = reader.read_joints(0) {
            for (idx, val) in joints.into_u16().enumerate() {
                vertices[idx].joints[0] = val[0] as u8;
                vertices[idx].joints[1] = val[1] as u8;
            }
        }

        if let Some(weights) = reader.read_weights(0) {
            for (idx, val) in weights.into_u8().enumerate() {
                vertices[idx].weights[0] = val[0] as u8;
                vertices[idx].weights[1] = val[1] as u8;
            }
        }

        // TODO: handle non-indexed GLTF files
        let indices: Vec<u16> = reader.read_indices().unwrap().into_u32().map(|x| x as u16).collect();

        let topology = match primitive.mode() {
            gltf::mesh::Mode::Triangles => gl::TRIANGLES,
            gltf::mesh::Mode::TriangleStrip => gl::TRIANGLE_STRIP,
            gltf::mesh::Mode::Lines => gl::LINES,
            gltf::mesh::Mode::LineStrip => gl::LINE_STRIP,
            _ => panic!("Unsupported GLTF primitive: {:?}", primitive.mode())
        };

        let mut mesh = MeshPart::new(topology);
        mesh.material_index = primitive.material().index().unwrap();
        mesh.vertices = vertices;
        mesh.indices = indices;
        mesh.apply();

        mesh
    }

    pub fn apply(self: &mut Self) {
        let vtx_len = (self.vertices.len() * size_of::<MeshVertex>()) as isize;
        let idx_len = (self.indices.len() * size_of::<u16>()) as isize;

        if let Some((vtx_buf, idx_buf)) = &mut self.buffers {
            if vtx_buf.size() < vtx_len {
                vtx_buf.resize(vtx_len);
            }

            if idx_buf.size() < idx_len {
                idx_buf.resize(idx_len);
            }

            vtx_buf.set_data(0, &self.vertices);
            idx_buf.set_data(0, &self.indices);
        }
        else {
            let mut vtx_buf = Buffer::new(vtx_len);
            let mut idx_buf = Buffer::new(idx_len);

            vtx_buf.set_data(0, &self.vertices);
            idx_buf.set_data(0, &self.indices);

            self.buffers = Some((vtx_buf, idx_buf));
        }
    }
}

pub struct MeshGroup {
    pub parts: Vec<MeshPart>
}

impl MeshGroup {
    pub fn from_gltf(mesh: Mesh, buffers: &[Data]) -> MeshGroup {
        let parts = mesh.primitives().map(|x| {
            MeshPart::from_gltf(x, buffers)
        }).collect();

        MeshGroup {
            parts
        }
    }
}

pub struct ModelNode {
    pub mesh_index: isize,
    pub num_children: usize,
    pub transform: Matrix4x4,
}

pub struct Model {
    pub meshes: Vec<MeshGroup>,
    pub materials: Vec<Arc<Material>>,
    pub nodes: Vec<ModelNode>,
}

impl Model {
    fn unpack_node(node: &Node, nodes: &mut Vec<ModelNode>) {
        let mesh_index = if let Some(mesh) = node.mesh() {
            mesh.index() as isize
        }
        else {
            -1
        };

        let transform = Matrix4x4 { m: node.transform().matrix() }.transposed();

        nodes.push(ModelNode { mesh_index, num_children: node.children().count(), transform });

        for child in node.children() {
            Model::unpack_node(&child, nodes);
        }
    }

    pub fn from_gltf(gltf: &Document, buffers: &[Data], material_path: &str) -> Model {
        // load meshes
        let meshes: Vec<MeshGroup> = gltf.meshes().map(|x| {
            MeshGroup::from_gltf(x, buffers)
        }).collect();

        // load materials
        let materials: Vec<Arc<Material>> = gltf.materials().map(|x| {
            let mat_path = format!("{}/{}.toml", material_path, x.name().unwrap());
            load_material(mat_path.as_str()).unwrap()
        }).collect();

        // unpack nodes
        let scene = gltf.default_scene().unwrap();
        let mut nodes = Vec::new();

        for node in scene.nodes() {
            Model::unpack_node(&node, &mut nodes);
        }

        Model { meshes, materials, nodes }
    }
}