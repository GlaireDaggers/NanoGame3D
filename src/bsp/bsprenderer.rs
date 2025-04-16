use std::{collections::HashSet, mem::offset_of, sync::Arc};

use lazy_static::lazy_static;
use crate::{asset_loader::{load_material, load_shader}, gl_checked, graphics::{buffer::Buffer, material::{Material, TextureSampler}, shader::Shader, texture::{Texture, TextureFormat}}, math::{Matrix4x4, Vector2, Vector3, Vector4}, misc::Color32};
use super::{bspcommon::{aabb_aabb_intersects, aabb_frustum}, bspfile::{BspFile, Edge, StaticPropVertex, SURF_NODRAW, SURF_SKY, SURF_TRANS33, SURF_TRANS66}, bsplightmap::BspLightmap};

pub const NUM_CUSTOM_LIGHT_LAYERS: usize = 30;
pub const CUSTOM_LIGHT_LAYER_START: usize = 32;
pub const CUSTOM_LIGHT_LAYER_END: usize = CUSTOM_LIGHT_LAYER_START + NUM_CUSTOM_LIGHT_LAYERS;

lazy_static! {
    static ref LIGHTSTYLES: [Vec<f32>;12] = [
        make_light_table(b"m"),
        make_light_table(b"mmnmmommommnonmmonqnmmo"),
        make_light_table(b"abcdefghijklmnopqrstuvwxyzyxwvutsrqponmlkjihgfedcba"),
        make_light_table(b"mmmmmaaaaammmmmaaaaaabcdefgabcdefg"),
        make_light_table(b"mamamamamama"),
        make_light_table(b"mamamamamamajklmnopqrstuvwxyzyxwvutsrqponmlkj"),
        make_light_table(b"nmonqnmomnmomomno"),
        make_light_table(b"mmmaaaabcdefgmmmmaaaammmaamm"),
        make_light_table(b"mmmaaammmaaammmabcdefaaaammmmabcdefmmmaaaa"),
        make_light_table(b"aaaaaaaazzzzzzzz"),
        make_light_table(b"mmamammmmammamamaaamammma"),
        make_light_table(b"abcdefghijklmnopqrrqponmlkjihgfedcba"),
    ];
}

// convert Quake-style light animation table to float array ('a' is minimum light, 'z' is maximum light)
fn make_light_table(data: &[u8]) -> Vec<f32> {
    let mut output = vec![0.0;data.len()];

    for i in 0..data.len() {
        output[i] = (data[i] - 97) as f32 / 25.0;
    }

    output
}

fn unpack_face(bsp: &BspFile, textures: &BspMapTextures, light_styles: &[f32], face_idx: usize, edge_buffer: &mut Vec<Edge>, geo: &mut Vec<MapVertex>, index: &mut Vec<u16>, lm: &BspLightmap) {
    let face = &bsp.face_lump.faces[face_idx];
    let tex_idx = face.texture_info as usize;
    let tex_info = &bsp.tex_info_lump.textures[tex_idx];

    if tex_info.flags & SURF_NODRAW != 0 {
        return;
    }

    if tex_info.flags & SURF_SKY != 0 {
        return;
    }

    let mut col = Color32::new(255, 255, 255, 255);

    if tex_info.flags & SURF_TRANS33 != 0 {
        col.a = 85;
    }
    else if tex_info.flags & SURF_TRANS66 != 0 {
        col.a = 170;
    }

    let start_edge_idx = face.first_edge as usize;
    let end_edge_idx = start_edge_idx + (face.num_edges as usize);

    edge_buffer.clear();
    for face_edge in start_edge_idx..end_edge_idx {
        let edge_idx = bsp.face_edge_lump.edges[face_edge];
        let reverse = edge_idx < 0;

        let edge = bsp.edge_lump.edges[edge_idx.abs() as usize];

        if reverse {
            edge_buffer.push(Edge{ a: edge.b, b: edge.a });
        }
        else {
            edge_buffer.push(edge);
        }
    }

    let mut tex_min = Vector2::new(f32::INFINITY, f32::INFINITY);
    let mut tex_max = Vector2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);

    // calculate UVs
    for i in 0..edge_buffer.len() {
        let e = &edge_buffer[i];

        let pos_a = bsp.vertex_lump.vertices[e.a as usize];
        let pos_b = bsp.vertex_lump.vertices[e.b as usize];

        let tex_a = Vector2::new(
            pos_a.dot(tex_info.u_axis) + tex_info.u_offset,
            pos_a.dot(tex_info.v_axis) + tex_info.v_offset
        );

        let tex_b = Vector2::new(
            pos_b.dot(tex_info.u_axis) + tex_info.u_offset,
            pos_b.dot(tex_info.v_axis) + tex_info.v_offset
        );

        tex_min.x = tex_min.x.min(tex_a.x);
        tex_min.y = tex_min.y.min(tex_a.y);
        tex_min.x = tex_min.x.min(tex_b.x);
        tex_min.y = tex_min.y.min(tex_b.y);

        tex_max.x = tex_max.x.max(tex_a.x);
        tex_max.y = tex_max.y.max(tex_a.y);
        tex_max.x = tex_max.x.max(tex_b.x);
        tex_max.y = tex_max.y.max(tex_b.y);
    }

    let lm_regions = lm.results[face_idx];
    let mut lm_region_offsets = [Vector2::zero();4];
    let mut lm_region_scales = [Vector2::zero();4];

    // NOTE: half texel bias applied to edges to fix bilinear sampling artifacts
    for i in 0..4 {
        lm_region_offsets[i] = Vector2::new((lm_regions[i].x as f32 + 0.5) / lm.texture.width() as f32, (lm_regions[i].y as f32 + 0.5) / lm.texture.height() as f32);
        lm_region_scales[i] = Vector2::new((lm_regions[i].width as f32 - 1.0) / lm.texture.width() as f32, (lm_regions[i].height as f32 - 1.0) / lm.texture.height() as f32);
    }

    // build triangle fan out of edges (note: clockwise winding)
    let idx_start = geo.len();

    for i in 0..edge_buffer.len() {
        let pos = edge_buffer[i].a as usize;
        let pos = bsp.vertex_lump.vertices[pos];

        let tex = Vector2::new(
            pos.dot(tex_info.u_axis) + tex_info.u_offset,
            pos.dot(tex_info.v_axis) + tex_info.v_offset
        );

        let mut lm_uvs = [Vector3::zero();4];
        for i in 0..4 {
            let lm_uv = ((tex - tex_min) / (tex_max - tex_min) * lm_region_scales[i]) + lm_region_offsets[i];
            lm_uvs[i] = Vector3::new(lm_uv.x, lm_uv.y, light_styles[face.lightmap_styles[i] as usize]);
        }

        let mat = &textures.loaded_materials[tex_idx];
        let texture = &mat.texture["mainTexture"].texture;

        let sc = Vector2::new(1.0 / texture.width() as f32, 1.0 / texture.height() as f32);
        let tex = tex * sc;

        let pos = Vector4::new(pos.x, pos.y, pos.z, 1.0);

        let vtx = MapVertex::new(pos, tex, lm_uvs[0], lm_uvs[1], lm_uvs[2], lm_uvs[3], col);

        geo.push(vtx);
    }

    for i in 1..edge_buffer.len() - 1 {
        let idx0 = idx_start;
        let idx1 = idx_start + i;
        let idx2 = idx_start + i + 1;

        index.push(idx0 as u16);
        index.push(idx1 as u16);
        index.push(idx2 as u16);
    }
}

fn setup_vtx_arrays(position: u32, uv: u32, lm0: u32, lm1: u32, lm2: u32, lm3: u32, color: u32) {
    unsafe {
        gl::EnableVertexAttribArray(position);
        gl::EnableVertexAttribArray(uv);
        gl::EnableVertexAttribArray(lm0);
        gl::EnableVertexAttribArray(lm1);
        gl::EnableVertexAttribArray(lm2);
        gl::EnableVertexAttribArray(lm3);
        gl::EnableVertexAttribArray(color);
        gl::VertexAttribPointer(position, 4, gl::FLOAT, gl::FALSE, size_of::<MapVertex>() as i32, offset_of!(MapVertex, position) as *const _);
        gl::VertexAttribPointer(uv, 2, gl::FLOAT, gl::FALSE, size_of::<MapVertex>() as i32, offset_of!(MapVertex, uv) as *const _);
        gl::VertexAttribPointer(lm0, 3, gl::FLOAT, gl::FALSE, size_of::<MapVertex>() as i32, offset_of!(MapVertex, lm0) as *const _);
        gl::VertexAttribPointer(lm1, 3, gl::FLOAT, gl::FALSE, size_of::<MapVertex>() as i32, offset_of!(MapVertex, lm1) as *const _);
        gl::VertexAttribPointer(lm2, 3, gl::FLOAT, gl::FALSE, size_of::<MapVertex>() as i32, offset_of!(MapVertex, lm2) as *const _);
        gl::VertexAttribPointer(lm3, 3, gl::FLOAT, gl::FALSE, size_of::<MapVertex>() as i32, offset_of!(MapVertex, lm3) as *const _);
        gl::VertexAttribPointer(color, 4, gl::UNSIGNED_BYTE, gl::TRUE, size_of::<MapVertex>() as i32, offset_of!(MapVertex, color) as *const _);
    }
}

fn bind_lightmap(lm: &BspLightmap) {
    unsafe {
        gl::ActiveTexture(gl::TEXTURE1);

        gl_checked!{ gl::BindTexture(gl::TEXTURE_2D, lm.texture.handle()) }
        gl_checked!{ gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32) }
        gl_checked!{ gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32) }
        gl_checked!{ gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32) }
        gl_checked!{ gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32) }
    }
}

fn draw_geom_setup(material: &Material, model: Matrix4x4, viewproj: Matrix4x4) {
    unsafe {
        gl::FrontFace(gl::CW);
    }

    material.apply();

    material.shader.set_uniform_mat4("mvp", model * viewproj);
    material.shader.set_uniform_int("lmTexture", 1);
}

#[derive(Clone, Copy)]
pub struct MapVertex {
    pub position: Vector4,
    pub uv: Vector2,
    pub lm0: Vector3,
    pub lm1: Vector3,
    pub lm2: Vector3,
    pub lm3: Vector3,
    pub color: Color32,
}

pub struct BspMapTextures {
    loaded_materials: Vec<Arc<Material>>,
    sprop_materials: Vec<Arc<Material>>,
    opaque_meshes: Vec<usize>,
    transp_meshes: Vec<usize>,
}

impl BspMapTextures {
    pub fn new(bsp_file: &BspFile) -> BspMapTextures {
        // load unique textures
        let mut loaded_materials: Vec<Arc<Material>> = Vec::new();
        let mut sprop_materials: Vec<Arc<Material>> = Vec::new();

        let mut opaque_meshes: Vec<usize> = Vec::new();
        let mut transp_meshes: Vec<usize> = Vec::new();

        let map_shader = load_shader("content/shaders/map_shader.toml").unwrap();
        let mut err_mat = Material::new(map_shader);

        let mut err_tex = Texture::new(TextureFormat::RGBA8888, 2, 2, 1);
        err_tex.set_texture_data(0, &[
            Color32::new(255, 0, 255, 255), Color32::new(0, 0, 0, 255),
            Color32::new(0, 0, 0, 255), Color32::new(255, 0, 255, 255)
        ]);

        err_mat.texture.insert("mainTexture".to_string(), TextureSampler { texture: Arc::new(err_tex), filter: false, wrap_s: true, wrap_t: true });

        let err_mat = Arc::new(err_mat);

        for (i, tex_info) in bsp_file.tex_info_lump.textures.iter().enumerate() {
            let material = match load_material(format!("content/materials/{}.toml", &tex_info.texture_name).as_str()) {
                Ok(v) => v,
                Err(_) => err_mat.clone()
            };

            if material.transparent {
                transp_meshes.push(i);
            }
            else {
                opaque_meshes.push(i);
            }

            loaded_materials.push(material);
        }

        for mat_name in &bsp_file.sprop_materials_lump.materials {
            let material = match load_material(format!("content/models/{}_PROP.toml", mat_name).as_str()) {
                Ok(v) => v,
                Err(_) => err_mat.clone()
            };

            sprop_materials.push(material);
        }

        BspMapTextures {
            loaded_materials,
            sprop_materials,
            opaque_meshes,
            transp_meshes
        }
    }
}

impl MapVertex {
    pub fn new(position: Vector4, uv: Vector2, lm0: Vector3, lm1: Vector3, lm2: Vector3, lm3: Vector3, color: Color32) -> MapVertex {
        MapVertex {
            position,
            uv,
            lm0,
            lm1,
            lm2,
            lm3,
            color
        }
    }
}

struct ModelPart {
    tex_idx: usize,
    light_styles: [u8;4],
    needs_update: bool,
    idx_len: usize,
    geom: Vec<MapVertex>,
    vtx_buffer: Buffer,
    idx_buffer: Buffer
}

struct Model {
    parts: Vec<ModelPart>
}

pub struct BspMapModelRenderer {
    models: Vec<Model>
}

impl BspMapModelRenderer {
    pub fn new(bsp_file: &BspFile, textures: &BspMapTextures, lm: &BspLightmap) -> BspMapModelRenderer {
        let mut light_styles = [0.0;256];
        light_styles[0] = LIGHTSTYLES[0][0];

        let mut models = Vec::new();
        let mut edges = Vec::new();
        for i in 1..bsp_file.submodel_lump.submodels.len() {
            let model = &bsp_file.submodel_lump.submodels[i];
            let mut model_parts = Vec::new();

            let start_face_idx = model.first_face as usize;
            let end_face_idx: usize = start_face_idx + (model.num_faces as usize);

            for face_idx in start_face_idx..end_face_idx {
                let mut geom = Vec::new();
                let mut idx = Vec::new();

                let face = &bsp_file.face_lump.faces[face_idx];
                let tex_idx = face.texture_info as usize;

                unpack_face(bsp_file, textures, &light_styles, face_idx, &mut edges, &mut geom, &mut idx, lm);

                let mut vtx_buffer = Buffer::new((geom.len() * size_of::<MapVertex>()) as isize);
                vtx_buffer.set_data(0, &geom);

                let mut idx_buffer = Buffer::new((idx.len() * size_of::<u16>()) as isize);
                idx_buffer.set_data(0, &idx);

                // optimization: if a face only has a single light style of 0, we don't need to bother updating the vertices for lightmapping
                let needs_lm_update = face.lightmap_styles[0] == 0 && face.num_lightmaps == 1;

                model_parts.push(ModelPart { tex_idx, light_styles: face.lightmap_styles, geom, vtx_buffer, idx_buffer, idx_len: idx.len(), needs_update: needs_lm_update });
            }

            models.push(Model {
                parts: model_parts
            });
        }

        BspMapModelRenderer {
            models
        }
    }

    /// call each frame to update lightmap animation for a given set of visible models
    pub fn update(self: &mut BspMapModelRenderer, light_layers: &[f32;NUM_CUSTOM_LIGHT_LAYERS], models: &[usize], animation_time: f32) {
        let lightstyle_frame = (animation_time * 10.0) as usize;
        let mut light_styles = [0.0;256];

        for (idx, tbl) in LIGHTSTYLES.iter().enumerate() {
            light_styles[idx] = tbl[lightstyle_frame % tbl.len()];
        }

        for (idx, sc) in light_layers.iter().enumerate() {
            light_styles[idx + CUSTOM_LIGHT_LAYER_START] = *sc;
        }

        for idx in models {
            for part in &mut self.models[*idx].parts {
                if part.needs_update {
                    for vtx in part.geom.iter_mut() {
                        vtx.lm0.z = light_styles[part.light_styles[0] as usize];
                        vtx.lm1.z = light_styles[part.light_styles[1] as usize];
                        vtx.lm2.z = light_styles[part.light_styles[2] as usize];
                        vtx.lm3.z = light_styles[part.light_styles[3] as usize];
                    }
    
                    part.vtx_buffer.set_data(0, &part.geom);
                }
            }
        }
    }

    pub fn draw_model(self: &mut BspMapModelRenderer, transparent: bool, textures: &BspMapTextures, lm: &BspLightmap, model_idx: usize, model_transform: Matrix4x4, camera_viewproj: Matrix4x4) {
        let model = &self.models[model_idx];

        for part in &model.parts {
            let material = &textures.loaded_materials[part.tex_idx];
            if material.transparent == transparent {
                draw_geom_setup(material, model_transform, camera_viewproj);
                bind_lightmap(lm);

                let shader_position = material.shader.get_attribute_location("in_pos");
                let shader_uv = material.shader.get_attribute_location("in_uv");
                let shader_lm0 = material.shader.get_attribute_location("in_lm0");
                let shader_lm1 = material.shader.get_attribute_location("in_lm1");
                let shader_lm2 = material.shader.get_attribute_location("in_lm2");
                let shader_lm3 = material.shader.get_attribute_location("in_lm3");
                let shader_color = material.shader.get_attribute_location("in_col");

                unsafe {
                    gl_checked!{ gl::BindBuffer(gl::ARRAY_BUFFER, part.vtx_buffer.handle()) }
                    gl_checked!{ gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, part.idx_buffer.handle()) }
    
                    setup_vtx_arrays(shader_position, shader_uv, shader_lm0, shader_lm1, shader_lm2, shader_lm3, shader_color);
    
                    // draw geometry
                    gl_checked!{ gl::DrawElements(gl::TRIANGLES, part.idx_len as i32, gl::UNSIGNED_SHORT, 0 as *const _) }
                }
            }
        }
    }
}

struct StaticPropMesh {
    mat_idx: usize,
    topology: gl::types::GLenum,
    vtx_buffer: Buffer,
    idx_buffer: Buffer,
    num_indices: usize,
}

impl StaticPropVertex {
    pub fn setup_vtx_arrays(shader: &Shader) {
        let position = shader.get_attribute_location("in_position");
        let normal = shader.get_attribute_location("in_normal");
        let tangent = shader.get_attribute_location("in_tangent");
        let texcoord0 = shader.get_attribute_location("in_texcoord");
        let color = shader.get_attribute_location("in_color");

        unsafe {
            gl::EnableVertexAttribArray(position);
            gl::EnableVertexAttribArray(normal);
            gl::EnableVertexAttribArray(tangent);
            gl::EnableVertexAttribArray(texcoord0);
            gl::EnableVertexAttribArray(color);

            gl::VertexAttribPointer(position, 4, gl::FLOAT, gl::FALSE, size_of::<StaticPropVertex>() as i32, offset_of!(StaticPropVertex, position) as *const _);
            gl::VertexAttribPointer(normal, 4, gl::FLOAT, gl::FALSE, size_of::<StaticPropVertex>() as i32, offset_of!(StaticPropVertex, normal) as *const _);
            gl::VertexAttribPointer(tangent, 4, gl::FLOAT, gl::FALSE, size_of::<StaticPropVertex>() as i32, offset_of!(StaticPropVertex, tangent) as *const _);
            gl::VertexAttribPointer(texcoord0, 2, gl::FLOAT, gl::FALSE, size_of::<StaticPropVertex>() as i32, offset_of!(StaticPropVertex, texcoord) as *const _);
            gl::VertexAttribPointer(color, 4, gl::UNSIGNED_BYTE, gl::TRUE, size_of::<StaticPropVertex>() as i32, offset_of!(StaticPropVertex, color) as *const _);
        }
    }
}

pub struct BspMapRenderer {
    vis: Vec<bool>,
    prev_leaf: i32,
    mesh_vertices: Vec<Vec<MapVertex>>,
    mesh_indices: Vec<Vec<u16>>,
    visible_leaves: HashSet<usize>,
    drawn_faces: Vec<u32>,
    cur_frame: u32,
    vtx_buffers: Vec<Buffer>,
    idx_buffers: Vec<Buffer>,
    static_props: Vec<StaticPropMesh>,
}

impl BspMapRenderer {
    pub fn new(bsp_file: &BspFile) -> BspMapRenderer {
        let num_clusters = bsp_file.vis_lump.clusters.len();
        let num_leaves = bsp_file.leaf_lump.leaves.len();
        let num_textures = bsp_file.tex_info_lump.textures.len();
        let num_faces = bsp_file.face_lump.faces.len();

        let mut vtx_buffers = Vec::with_capacity(num_textures);
        let mut idx_buffers = Vec::with_capacity(num_textures);

        for _ in 0..num_textures {
            let vtx_buf = Buffer::new(1024 * size_of::<MapVertex>() as isize);
            let idx_buf = Buffer::new(1024 * size_of::<u16>() as isize);

            vtx_buffers.push(vtx_buf);
            idx_buffers.push(idx_buf);
        }

        let mut static_props = Vec::new();
        for sprop in &bsp_file.sprop_lump.props {
            let idx_start = sprop.first_index as usize;
            let idx_end = idx_start + sprop.num_indices as usize;
            let idx_slice = &bsp_file.sprop_indices_lump.indices[idx_start .. idx_end];

            let vtx_start = sprop.first_vertex as usize;
            let vtx_end = vtx_start + sprop.num_vertices as usize;
            let vtx_slice = &bsp_file.sprop_vertices_lump.vertices[vtx_start .. vtx_end];

            let mut idx_buffer = Buffer::new((idx_slice.len() * size_of::<u16>()) as isize);
            idx_buffer.set_data(0, idx_slice);

            let mut vtx_buffer = Buffer::new((vtx_slice.len() * size_of::<StaticPropVertex>()) as isize);
            vtx_buffer.set_data(0, vtx_slice);

            static_props.push(StaticPropMesh { topology: sprop.topology, mat_idx: sprop.material as usize, vtx_buffer, idx_buffer, num_indices: idx_slice.len() });
        }

        BspMapRenderer {
            vis: vec![false;num_clusters],
            visible_leaves: HashSet::with_capacity(num_leaves),
            mesh_vertices: vec![Vec::new();num_textures],
            mesh_indices: vec![Vec::new();num_textures],
            drawn_faces: vec![0;num_faces],
            cur_frame: 0,
            prev_leaf: -1,
            vtx_buffers,
            idx_buffers,
            static_props,
        }
    }

    fn update_leaf(bsp: &BspFile, leaf_index: usize, visible_clusters: &[bool], visible_leaves: &mut HashSet<usize>) {
        let leaf = &bsp.leaf_lump.leaves[leaf_index];
        if leaf.cluster == u16::MAX {
            return;
        }

        if visible_clusters[leaf.cluster as usize] {
            visible_leaves.insert(leaf_index);
        }
    }

    fn update_recursive(bsp: &BspFile, cur_node: i32, frustum: &[Vector4], visible_clusters: &[bool], visible_leaves: &mut HashSet<usize>) {
        if cur_node < 0 {
            Self::update_leaf(bsp, (-cur_node - 1) as usize, visible_clusters, visible_leaves);
            return;
        }

        let node = &bsp.node_lump.nodes[cur_node as usize];

        if !aabb_frustum(node._bbox_min, node._bbox_max, frustum) {
            return;
        }

        Self::update_recursive(bsp, node.front_child, frustum, visible_clusters, visible_leaves);
        Self::update_recursive(bsp, node.back_child, frustum, visible_clusters, visible_leaves);
    }

    /// Call each frame before rendering. Recalculates visible leaves & rebuilds geometry when necessary
    pub fn update(self: &mut Self, frustum: &[Vector4], anim_time: f32, light_layers: &[f32;NUM_CUSTOM_LIGHT_LAYERS], bsp: &BspFile, textures: &BspMapTextures, lm: &BspLightmap, position: Vector3) {
        self.cur_frame = self.cur_frame.wrapping_add(1);

        let leaf_index = bsp.calc_leaf_index(&position);
        let leaf = &bsp.leaf_lump.leaves[leaf_index as usize];

        let lightstyle_frame = (anim_time * 10.0) as usize;
        let mut light_styles = [0.0;256];

        for (idx, tbl) in LIGHTSTYLES.iter().enumerate() {
            light_styles[idx] = tbl[lightstyle_frame % tbl.len()];
        }

        for (idx, sc) in light_layers.iter().enumerate() {
            light_styles[idx + CUSTOM_LIGHT_LAYER_START] = *sc;
        }

        if leaf_index != self.prev_leaf {
             // unpack new cluster's visibility info
            self.prev_leaf = leaf_index;
            
            self.vis.fill(false);
            if leaf.cluster != u16::MAX {
                bsp.vis_lump.unpack_vis(leaf.cluster as usize, &mut self.vis);
            }
        }

        self.visible_leaves.clear();
        Self::update_recursive(bsp, 0, frustum, &self.vis, &mut self.visible_leaves);

        // build geometry for visible leaves
        for m in &mut self.mesh_vertices {
            m.clear();
        }

        for idx in &mut self.mesh_indices {
            idx.clear();
        }

        let mut edges: Vec<Edge> = Vec::new();

        for i in &self.visible_leaves {
            let leaf = &bsp.leaf_lump.leaves[*i];
            let start_face_idx = leaf.first_leaf_face as usize;
            let end_face_idx: usize = start_face_idx + (leaf.num_leaf_faces as usize);

            for leaf_face in start_face_idx..end_face_idx {
                let face_idx = bsp.leaf_face_lump.faces[leaf_face] as usize;

                if self.drawn_faces[face_idx] == self.cur_frame {
                    continue;
                }

                self.drawn_faces[face_idx] = self.cur_frame;

                let face = &bsp.face_lump.faces[face_idx];
                let tex_idx = face.texture_info as usize;
                unpack_face(bsp, textures, &light_styles, face_idx, &mut edges, &mut self.mesh_vertices[tex_idx], &mut self.mesh_indices[tex_idx], lm);
            }
        }

        // upload geometry data
        for i in 0..self.vtx_buffers.len() {
            if self.mesh_indices[i].len() > 0 {
                let vtx_buf_size = (self.mesh_vertices[i].len() * size_of::<MapVertex>()) as isize;
                let idx_buf_size = (self.mesh_indices[i].len() * size_of::<u16>()) as isize;

                if self.vtx_buffers[i].size() < vtx_buf_size {
                    self.vtx_buffers[i].resize(vtx_buf_size);
                }

                if self.idx_buffers[i].size() < idx_buf_size {
                    self.idx_buffers[i].resize(idx_buf_size);
                }

                self.vtx_buffers[i].set_data(0, &self.mesh_vertices[i]);
                self.idx_buffers[i].set_data(0, &self.mesh_indices[i]);
            }
        }
    }

    fn get_bounds_corners(center: Vector3, extents: Vector3) -> [Vector3;8] {
        [
            center + Vector3::new(-extents.x, -extents.y, -extents.z),
            center + Vector3::new( extents.x, -extents.y, -extents.z),
            center + Vector3::new(-extents.x,  extents.y, -extents.z),
            center + Vector3::new( extents.x,  extents.y, -extents.z),
            center + Vector3::new(-extents.x, -extents.y,  extents.z),
            center + Vector3::new( extents.x, -extents.y,  extents.z),
            center + Vector3::new(-extents.x,  extents.y,  extents.z),
            center + Vector3::new( extents.x,  extents.y,  extents.z),
        ]
    }

    fn check_vis_leaf(self: &Self, bsp: &BspFile, leaf_index: usize, center: Vector3, extents: Vector3) -> bool {
        if !self.visible_leaves.contains(&leaf_index) {
            return false;
        }

        let min = center - extents;
        let max = center + extents;

        let leaf = &bsp.leaf_lump.leaves[leaf_index];

        return aabb_aabb_intersects(min, max, leaf.bbox_min, leaf.bbox_max);
    }

    fn check_vis_recursive(self: &Self, bsp: &BspFile, node_index: i32, center: Vector3, extents: Vector3, corners: &[Vector3;8]) -> bool {
        if node_index < 0 {
            return self.check_vis_leaf(bsp, (-node_index - 1) as usize, center, extents);
        }

        let node = &bsp.node_lump.nodes[node_index as usize];
        let split_plane = &bsp.plane_lump.planes[node.plane as usize];

        let mut dmin = f32::MAX;
        let mut dmax = f32::MIN;

        for i in 0..8 {
            let d = corners[i].dot(split_plane.normal) - split_plane.distance;

            if d < dmin {
                dmin = d;
            }

            if d > dmax {
                dmax = d;
            }
        }

        if dmax >= 0.0 {
            if self.check_vis_recursive(bsp, node.front_child, center, extents, corners) {
                return true;
            }
        }

        if dmin <= 0.0 {
            if self.check_vis_recursive(bsp, node.back_child, center, extents, corners) {
                return true;
            }
        }

        return false;
    }

    pub fn check_vis(self: &Self, bsp: &BspFile, center: Vector3, extents: Vector3) -> bool {
        let corners = Self::get_bounds_corners(center, extents);
        return self.check_vis_recursive(bsp, 0, center, extents, &corners);
    }

    pub fn is_leaf_visible(self: &Self, leaf_index: usize) -> bool {
        return self.visible_leaves.contains(&leaf_index);
    }

    pub fn draw_opaque(self: &mut Self, textures: &BspMapTextures, lm: &BspLightmap, animation_time: f32, camera_viewproj: Matrix4x4) {
        for i in &textures.opaque_meshes {
            if self.mesh_indices[*i].len() > 0 {
                let vtx_buf = &self.vtx_buffers[*i];
                let idx_buf = &self.idx_buffers[*i];

                let material = &textures.loaded_materials[*i];
                draw_geom_setup(&material, Matrix4x4::identity(), camera_viewproj);
                bind_lightmap(lm);

                let shader_position = material.shader.get_attribute_location("in_pos");
                let shader_uv = material.shader.get_attribute_location("in_uv");
                let shader_lm0 = material.shader.get_attribute_location("in_lm0");
                let shader_lm1 = material.shader.get_attribute_location("in_lm1");
                let shader_lm2 = material.shader.get_attribute_location("in_lm2");
                let shader_lm3 = material.shader.get_attribute_location("in_lm3");
                let shader_color = material.shader.get_attribute_location("in_col");

                material.shader.set_uniform_float("time", animation_time);

                unsafe {
                    gl_checked!{ gl::BindBuffer(gl::ARRAY_BUFFER, vtx_buf.handle()) }
                    gl_checked!{ gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, idx_buf.handle()) }
    
                    setup_vtx_arrays(shader_position, shader_uv, shader_lm0, shader_lm1, shader_lm2, shader_lm3, shader_color);

                    // draw geometry
                    gl_checked!{ gl::DrawElements(gl::TRIANGLES, self.mesh_indices[*i].len() as i32, gl::UNSIGNED_SHORT, 0 as *const _) }
                }
            }
        }

        for prop in &self.static_props {
            let mat = &textures.sprop_materials[prop.mat_idx];
            if mat.transparent == false {
                draw_geom_setup(&mat, Matrix4x4::identity(), camera_viewproj);
                mat.shader.set_uniform_float("time", animation_time);

                unsafe {
                    gl::FrontFace(gl::CCW);
                    
                    gl_checked!{ gl::BindBuffer(gl::ARRAY_BUFFER, prop.vtx_buffer.handle()) }
                    gl_checked!{ gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, prop.idx_buffer.handle()) }

                    StaticPropVertex::setup_vtx_arrays(&mat.shader);

                    // draw geometry
                    gl_checked!{ gl::DrawElements(prop.topology, prop.num_indices as i32, gl::UNSIGNED_SHORT, 0 as *const _) }
                }
            }
        }
    }

    pub fn draw_transparent(self: &mut Self, textures: &BspMapTextures, lm: &BspLightmap, animation_time: f32, camera_viewproj: Matrix4x4) {
        for i in &textures.transp_meshes {
            if self.mesh_indices[*i].len() > 0 {
                let vtx_buf = &self.vtx_buffers[*i];
                let idx_buf = &self.idx_buffers[*i];

                let material = &textures.loaded_materials[*i];
                draw_geom_setup(&material, Matrix4x4::identity(), camera_viewproj);
                bind_lightmap(lm);

                let shader_position = material.shader.get_attribute_location("in_pos");
                let shader_uv = material.shader.get_attribute_location("in_uv");
                let shader_lm0 = material.shader.get_attribute_location("in_lm0");
                let shader_lm1 = material.shader.get_attribute_location("in_lm1");
                let shader_lm2 = material.shader.get_attribute_location("in_lm2");
                let shader_lm3 = material.shader.get_attribute_location("in_lm3");
                let shader_color = material.shader.get_attribute_location("in_col");

                material.shader.set_uniform_float("time", animation_time);

                unsafe {
                    gl_checked!{ gl::BindBuffer(gl::ARRAY_BUFFER, vtx_buf.handle()) }
                    gl_checked!{ gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, idx_buf.handle()) }
    
                    setup_vtx_arrays(shader_position, shader_uv, shader_lm0, shader_lm1, shader_lm2, shader_lm3, shader_color);

                    // draw geometry
                    gl_checked!{ gl::DrawElements(gl::TRIANGLES, self.mesh_indices[*i].len() as i32, gl::UNSIGNED_SHORT, 0 as *const _) }
                }
            }
        }

        for prop in &self.static_props {
            let mat = &textures.sprop_materials[prop.mat_idx];
            if mat.transparent {
                draw_geom_setup(&mat, Matrix4x4::identity(), camera_viewproj);
                mat.shader.set_uniform_float("time", animation_time);

                unsafe {
                    gl::FrontFace(gl::CCW);
                    
                    gl_checked!{ gl::BindBuffer(gl::ARRAY_BUFFER, prop.vtx_buffer.handle()) }
                    gl_checked!{ gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, prop.idx_buffer.handle()) }

                    StaticPropVertex::setup_vtx_arrays(&mat.shader);

                    // draw geometry
                    gl_checked!{ gl::DrawElements(prop.topology, prop.num_indices as i32, gl::UNSIGNED_SHORT, 0 as *const _) }
                }
            }
        }
    }
}