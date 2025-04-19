use std::{collections::HashMap, mem::offset_of, sync::Arc};

use gltf::{buffer::Data, Animation, Document, Mesh, Node, Primitive};

use crate::{asset_loader::load_material, math::{Matrix4x4, Quaternion, Vector2, Vector3, Vector4}, misc::Color32};

use super::{anim::AnimationCurve, buffer::Buffer, material::Material, shader::Shader};

#[derive(Default, Clone, Copy)]
pub struct MeshVertex {
    pub position: Vector4,
    pub normal: Vector4,
    pub tangent: Vector4,
    pub texcoord0: Vector2,
    pub texcoord1: Vector2,
    pub color: Color32,
    pub joints: [u8;4],
    pub weights: [u8;4],
}

impl MeshVertex {
    pub fn setup_vtx_arrays(shader: &Shader) {
        let position = shader.get_attribute_location("in_position");
        let normal = shader.get_attribute_location("in_normal");
        let tangent = shader.get_attribute_location("in_tangent");
        let texcoord0 = shader.get_attribute_location("in_texcoord0");
        let texcoord1 = shader.get_attribute_location("in_texcoord1");
        let color = shader.get_attribute_location("in_color");

        unsafe {
            gl::EnableVertexAttribArray(position);
            gl::EnableVertexAttribArray(normal);
            gl::EnableVertexAttribArray(tangent);
            gl::EnableVertexAttribArray(texcoord0);
            gl::EnableVertexAttribArray(texcoord1);
            gl::EnableVertexAttribArray(color);

            gl::VertexAttribPointer(position, 4, gl::FLOAT, gl::FALSE, size_of::<MeshVertex>() as i32, offset_of!(MeshVertex, position) as *const _);
            gl::VertexAttribPointer(normal, 4, gl::FLOAT, gl::FALSE, size_of::<MeshVertex>() as i32, offset_of!(MeshVertex, normal) as *const _);
            gl::VertexAttribPointer(tangent, 4, gl::FLOAT, gl::FALSE, size_of::<MeshVertex>() as i32, offset_of!(MeshVertex, tangent) as *const _);
            gl::VertexAttribPointer(texcoord0, 2, gl::FLOAT, gl::FALSE, size_of::<MeshVertex>() as i32, offset_of!(MeshVertex, texcoord0) as *const _);
            gl::VertexAttribPointer(texcoord1, 2, gl::FLOAT, gl::FALSE, size_of::<MeshVertex>() as i32, offset_of!(MeshVertex, texcoord1) as *const _);
            gl::VertexAttribPointer(color, 4, gl::UNSIGNED_BYTE, gl::TRUE, size_of::<MeshVertex>() as i32, offset_of!(MeshVertex, color) as *const _);
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

        for pos in positions {
            let vtx = MeshVertex {
                position: Vector4::new(pos[0], pos[1], pos[2], 1.0),
                normal: Vector4::zero(),
                tangent: Vector4::zero(),
                texcoord0: Vector2::zero(),
                texcoord1: Vector2::zero(),
                color: Color32::new(255, 255, 255, 255),
                joints: [0, 0, 0, 0],
                weights: [0, 0, 0, 0],
            };

            vertices.push(vtx);
        }

        if let Some(normals) = reader.read_normals() {
            for (idx, val) in normals.enumerate() {
                vertices[idx].normal = Vector4::new(val[0], val[1], val[2], 0.0);
            }
        }

        if let Some(tangents) = reader.read_tangents() {
            for (idx, val) in tangents.enumerate() {
                vertices[idx].tangent = Vector4::new(val[0], val[1], val[2], 0.0);
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
                vertices[idx].joints[2] = val[2] as u8;
                vertices[idx].joints[3] = val[3] as u8;
            }
        }

        if let Some(weights) = reader.read_weights(0) {
            for (idx, val) in weights.into_u8().enumerate() {
                #[cfg(feature = "two_bone_per_vertex")]
                {
                    // sort weights
                    let mut weights = val.iter().zip(vertices[idx].joints).collect::<Vec<_>>();
                    weights.sort_by_key(|x| x.0);

                    // pick the two highest weights, re-normalize
                    let weight0 = *weights[3].0 as f32;
                    let weight1 = *weights[2].0 as f32;
                    let weight_total = weight0 + weight1;
                    let weight0 = weight0 / weight_total;
                    let weight1 = weight1 / weight_total;

                    vertices[idx].weights[0] = (weight0 * 255.0) as u8;
                    vertices[idx].weights[1] = (weight1 * 255.0) as u8;

                    vertices[idx].joints[0] = weights[3].1;
                    vertices[idx].joints[1] = weights[2].1;
                }

                #[cfg(not(feature = "two_bone_per_vertex"))]
                {
                    vertices[idx].weights[0] = val[0] as u8;
                    vertices[idx].weights[1] = val[1] as u8;
                    vertices[idx].weights[2] = val[2] as u8;
                    vertices[idx].weights[3] = val[3] as u8;
                }
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
    pub skin_index: isize,
    pub num_children: usize,
    pub transform: Matrix4x4,
    pub rest_pos: Vector3,
    pub rest_rot: Quaternion,
    pub rest_scale: Vector3,
}

#[derive(Default)]
pub struct ModelAnimationChannels {
    pub translation: Option<AnimationCurve<Vector3>>,
    pub rotation: Option<AnimationCurve<Quaternion>>,
    pub scale: Option<AnimationCurve<Vector3>>,
}

pub struct ModelAnimationClip {
    pub name: String,
    pub duration: f32,
    pub channels: HashMap<usize, ModelAnimationChannels>
}

impl ModelAnimationClip {
    pub fn from_gltf(buffers: &[Data], nodemap: &HashMap<usize, usize>, anim: &Animation) -> Self {
        let mut channels: HashMap<usize, ModelAnimationChannels> = HashMap::new();

        let mut duration: f32 = 0.0;

        for ch in anim.channels() {
            let target_node = nodemap[&ch.target().node().index()];
            if !channels.contains_key(&target_node) {
                channels.insert(target_node, ModelAnimationChannels::default());
            }
            let target_channels = channels.get_mut(&target_node).unwrap();

            let reader = ch.reader(|buffer| Some(&buffers[buffer.index()]));
            let keyframe_timestamps = if let Some(inputs) = reader.read_inputs() {
                match inputs {
                    gltf::accessor::Iter::Standard(item_iter) => {
                        let times: Vec<f32> = item_iter.collect();
                        duration = duration.max(times[times.len() - 1]);
                        times
                    },
                    gltf::accessor::Iter::Sparse(_) => todo!(),
                }
            }
            else {
                continue;
            };

            if let Some(outputs) = reader.read_outputs() {
                match outputs {
                    gltf::animation::util::ReadOutputs::Translations(iter) => {
                        let keyframes: Vec<Vector3> = iter.map(|v| Vector3::new(v[0], v[1], v[2])).collect();
                        target_channels.translation = Some(AnimationCurve::<Vector3>::from_gltf(ch.sampler().interpolation(), &keyframe_timestamps, &keyframes));
                    },
                    gltf::animation::util::ReadOutputs::Rotations(rotations) => {
                        let keyframes: Vec<Quaternion> = rotations.into_f32().map(|v| Quaternion::new(v[0], v[1], v[2], v[3])).collect();
                        target_channels.rotation = Some(AnimationCurve::<Quaternion>::from_gltf(ch.sampler().interpolation(), &keyframe_timestamps, &keyframes));
                    },
                    gltf::animation::util::ReadOutputs::Scales(iter) => {
                        let keyframes: Vec<Vector3> = iter.map(|v| Vector3::new(v[0], v[1], v[2])).collect();
                        target_channels.scale = Some(AnimationCurve::<Vector3>::from_gltf(ch.sampler().interpolation(), &keyframe_timestamps, &keyframes));
                    },
                    gltf::animation::util::ReadOutputs::MorphTargetWeights(_) => {
                        continue;
                    },
                }
            }
        }

        Self { name: anim.name().unwrap().to_string(), duration, channels }
    }
}

pub struct ModelSkinJoint {
    pub inv_bind_xform: Matrix4x4,
    pub node: usize,
}

pub struct ModelSkin {
    pub joints: Vec<ModelSkinJoint>,
}

pub struct Model {
    pub root_transform: Matrix4x4,
    pub meshes: Vec<MeshGroup>,
    pub materials: Vec<Arc<Material>>,
    pub nodes: Vec<ModelNode>,
    pub animations: Vec<ModelAnimationClip>,
    pub skins: Vec<ModelSkin>
}

impl Model {
    fn unpack_node(node: &Node, nodes: &mut Vec<ModelNode>, nodemap: &mut HashMap<usize, usize>) {
        let mesh_index = if let Some(mesh) = node.mesh() {
            mesh.index() as isize
        }
        else {
            -1
        };

        let skin_index = if let Some(skin) = node.skin() {
            skin.index() as isize
        }
        else {
            -1
        };

        let trs = node.transform().decomposed();
        let transform = Matrix4x4 { m: node.transform().matrix() };

        nodemap.insert(node.index(), nodes.len());

        nodes.push(ModelNode {
            mesh_index,
            skin_index,
            num_children: node.children().count(),
            transform,
            rest_pos: Vector3::new(trs.0[0], trs.0[1], trs.0[2]),
            rest_rot: Quaternion::new(trs.1[0], trs.1[1], trs.1[2], trs.1[3]),
            rest_scale: Vector3::new(trs.2[0], trs.2[1], trs.2[2]),
        });

        for child in node.children() {
            Model::unpack_node(&child, nodes, nodemap);
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
        let mut nodemap = HashMap::new();

        for node in scene.nodes() {
            Model::unpack_node(&node, &mut nodes, &mut nodemap);
        }

        let animations: Vec<ModelAnimationClip> = gltf.animations().map(|x| {
            ModelAnimationClip::from_gltf(buffers, &nodemap, &x)
        }).collect();

        let skins: Vec<ModelSkin> = gltf.skins().map(|x| {
            let reader = x.reader(|buffer| Some(&buffers[buffer.index()]));
            let inv_bind_matrices = reader.read_inverse_bind_matrices().unwrap();
            let joints = x.joints();

            let skin_joints: Vec<ModelSkinJoint> = inv_bind_matrices.zip(joints).map(|x| {
                ModelSkinJoint {
                    inv_bind_xform: Matrix4x4 { m: x.0 },
                    node: nodemap[&x.1.index()]
                }
            }).collect();

            ModelSkin { joints: skin_joints }
        }).collect();

        // GLTF -> NG3D: (.x, -.z, .y)

        Model {
            root_transform: Matrix4x4 { m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, -1.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 1.0]
            ]},
            meshes,
            materials,
            nodes,
            animations,
            skins
        }
    }

    pub fn get_animation_id(self: &Self, name: &str) -> Result<usize, ()> {
        for (idx, anim) in self.animations.iter().enumerate() {
            if anim.name == name {
                return Ok(idx);
            }
        }

        return Err(());
    }
}