use rect_packer::{Packer, Rect};
use crate::{graphics::texture::{Texture, TextureFormat}, math::Vector2};

use super::bspfile::{BspFile, SURF_NOLM};

const LM_SIZE: i32 = 1024;
const LM_MAX_FACE_SIZE: usize = 16;

pub struct BspLightmap {
    pub texture: Texture,
    pub results: Vec<[Rect;4]>
}

impl BspLightmap {
    pub fn new(bsp: &BspFile) -> BspLightmap {
        let packer_config = rect_packer::Config {
            width: LM_SIZE,
            height: LM_SIZE,
            border_padding: 0,
            rectangle_padding: 0
        };

        let mut packer = Packer::new(packer_config);
        let mut results = Vec::with_capacity(bsp.face_lump.faces.len());
        let mut texture = Texture::new(TextureFormat::RGBA8888, LM_SIZE, LM_SIZE, 1);

        // iterate each face in the BSP file
        for face in &bsp.face_lump.faces {
            let tex_idx = face.texture_info;
            let tex_info = &bsp.tex_info_lump.textures[tex_idx as usize];

            let mut lm_rects = [Rect::new(0, 0, 0, 0);4];

            if tex_info.flags & SURF_NOLM != 0 {
                results.push(lm_rects);
                continue;
            }

            if face.num_lightmaps == 0 {
                results.push(lm_rects);
                continue;
            }

            let start_edge_idx = face.first_edge as usize;
            let end_edge_idx = start_edge_idx + (face.num_edges as usize);

            // calculate lightmap UVs
            let mut tex_min = Vector2::new(f32::INFINITY, f32::INFINITY);
            let mut tex_max = Vector2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
            
            for face_edge in start_edge_idx..end_edge_idx {
                let edge_idx = bsp.face_edge_lump.edges[face_edge];
                let edge = bsp.edge_lump.edges[edge_idx.abs() as usize];

                let pos_a = bsp.vertex_lump.vertices[edge.a as usize];
                let pos_b = bsp.vertex_lump.vertices[edge.b as usize];

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

            let lm_size_x = ((tex_max.x / LM_MAX_FACE_SIZE as f32).ceil() - (tex_min.x / LM_MAX_FACE_SIZE as f32).floor() + 1.0).trunc() as usize;
            let lm_size_y = ((tex_max.y / LM_MAX_FACE_SIZE as f32).ceil() - (tex_min.y / LM_MAX_FACE_SIZE as f32).floor() + 1.0).trunc() as usize;

            let lm_size_x = lm_size_x.clamp(1, LM_MAX_FACE_SIZE);
            let lm_size_y = lm_size_y.clamp(1, LM_MAX_FACE_SIZE);

            let lm_slice_len = lm_size_x * lm_size_y;

            // a face can have up to 4 lightmaps associated with it, which each need to be packed into the atlas separately
            for i in 0..face.num_lightmaps {
                // pack rect into lightmap
                if let Some(rect) = packer.pack(lm_size_x as i32, lm_size_y as i32, false) {
                    lm_rects[i] = rect;

                    // upload to texture
                    let slice_start = (face.lightmap_offset / 3) as usize + (i * lm_slice_len);
                    let slice_end = slice_start + lm_slice_len;
                    let lm_slice = &bsp.lm_lump.lm[slice_start..slice_end];

                    texture.set_texture_data_region(0, rect.x, rect.y, rect.width, rect.height, lm_slice);
                }
                else {
                    panic!("Exhausted space in lightmap atlas!!");
                }
            }

            results.push(lm_rects);
        }
        
        BspLightmap {
            texture,
            results
        }
    }
}