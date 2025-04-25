use std::{mem::offset_of, sync::Arc};

use imgui::{DrawCmd, DrawVert};

use crate::{asset_loader::load_material, graphics::{buffer::Buffer, material::Material, shader::Shader, texture::{Texture, TextureFormat}}, math::{Matrix4x4, Vector3}};

pub struct Renderer {
    material: Arc<Material>,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    font_atlas: Texture,
}

fn setup_vtx_arrays(shader: &Arc<Shader>) {
    let position = shader.get_attribute_location("in_position");
    let texcoord = shader.get_attribute_location("in_texcoord");
    let color = shader.get_attribute_location("in_color");

    unsafe {
        gl::EnableVertexAttribArray(position);
        gl::EnableVertexAttribArray(texcoord);
        gl::EnableVertexAttribArray(color);

        gl::VertexAttribPointer(position, 2, gl::FLOAT, gl::FALSE, size_of::<DrawVert>() as i32, offset_of!(DrawVert, pos) as *const _);
        gl::VertexAttribPointer(texcoord, 2, gl::FLOAT, gl::FALSE, size_of::<DrawVert>() as i32, offset_of!(DrawVert, uv) as *const _);
        gl::VertexAttribPointer(color, 4, gl::UNSIGNED_BYTE, gl::TRUE, size_of::<DrawVert>() as i32, offset_of!(DrawVert, col) as *const _);
    }
}

impl Renderer {
    pub fn new(imgui: &mut imgui::Context) -> Renderer {
        let material = load_material("content/materials/misc/imgui.mat.ron").unwrap();

        let atlas = imgui.fonts();
        let atlas_tex = atlas.build_rgba32_texture();

        let mut font_atlas = Texture::new(TextureFormat::RGBA8888, atlas_tex.width as i32, atlas_tex.height as i32, 1);
        font_atlas.set_texture_data(0, atlas_tex.data);

        atlas.tex_id = (font_atlas.handle() as usize).into();

        Renderer {
            material,
            vertex_buffer: Buffer::new((16 * size_of::<DrawVert>()) as isize),
            index_buffer: Buffer::new((16 * size_of::<u16>()) as isize),
            font_atlas
        }
    }

    pub fn render(&mut self, imgui: &mut imgui::Context) {
        let [width, height] = imgui.io().display_size;
        let [scale_w, scale_h] = imgui.io().display_framebuffer_scale;

        let fb_width = width * scale_w;
        let fb_height = height * scale_h;

        let matrix = Matrix4x4::scale(Vector3::new(2.0 / fb_width, 2.0 / -fb_height, 1.0))
            * Matrix4x4::translation(Vector3::new(-1.0, 1.0, 0.0));
        
        self.material.apply();
        self.material.shader.resource.set_uniform_mat4("mvp", matrix);

        unsafe {
            gl::Enable(gl::SCISSOR_TEST);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.font_atlas.handle());

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

            self.material.shader.resource.set_uniform_int("mainTexture", 0);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer.handle());
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer.handle());

            setup_vtx_arrays(&self.material.shader.resource);
        }

        let draw_data = imgui.render();

        if draw_data.draw_lists_count() > 0 {
            for draw_list in draw_data.draw_lists() {
                let vtx_buffer = draw_list.vtx_buffer();
                let idx_buffer = draw_list.idx_buffer();
    
                // orphan buffers
                self.vertex_buffer.resize((vtx_buffer.len() * size_of::<DrawVert>()) as isize);
                self.index_buffer.resize((idx_buffer.len() * size_of::<u16>()) as isize);
    
                // upload new data
                self.vertex_buffer.set_data(0, vtx_buffer);
                self.index_buffer.set_data(0, idx_buffer);
    
                // process draw commands
                for cmd in draw_list.commands() {
                    match cmd {
                        DrawCmd::Elements { count, cmd_params } => unsafe {
                            let [x, y, z, w] = cmd_params.clip_rect;
    
                            gl::BindTexture(gl::TEXTURE_2D, cmd_params.texture_id.id() as u32);
                            gl::Scissor((x * scale_w) as i32,
                                (fb_height - w * scale_h) as i32,
                                ((z - x) * scale_w) as i32,
                                ((w - y) * scale_h) as i32);
    
                            gl::DrawElements(gl::TRIANGLES, count as i32, gl::UNSIGNED_SHORT, (cmd_params.idx_offset * size_of::<u16>()) as *const _);
                        },
                        DrawCmd::ResetRenderState => {
                            unimplemented!()
                        },
                        DrawCmd::RawCallback { .. } => {
                            unimplemented!()
                        }
                    }
                }
            }
        }

        unsafe {
            gl::Disable(gl::SCISSOR_TEST);
        }
    }
}