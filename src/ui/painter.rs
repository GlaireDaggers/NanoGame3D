use std::mem::offset_of;

use crate::{asset_loader::{load_material, MaterialHandle, ShaderHandle}, graphics::{buffer::Buffer, texture::Texture}, math::{Matrix4x4, Vector2, Vector3}, misc::{Color32, Rectangle}};

struct UiVertex {
    pos: Vector2,
    uv: Vector2,
    col: Color32,
}

struct UiBatch<'a> {
    pub vertices: Vec<UiVertex>,
    pub indices: Vec<u16>,
    pub texture: &'a Texture,
}

pub struct UiPaintPass<'a> {
    vtx_buffer: &'a mut Buffer,
    idx_buffer: &'a mut Buffer,
    max_quads: usize,
    batch: Option<UiBatch<'a>>,
    material: &'a MaterialHandle,
    window_size: (u32, u32),
}

pub struct UiPainter {
    vtx_buffer: Buffer,
    idx_buffer: Buffer,
    max_quads: usize,
    material: MaterialHandle,
}

fn setup_vtx_arrays(shader: &ShaderHandle) {
    let position = shader.get_attribute_location("in_position");
    let texcoord = shader.get_attribute_location("in_texcoord");
    let color = shader.get_attribute_location("in_color");

    unsafe {
        gl::EnableVertexAttribArray(position);
        gl::EnableVertexAttribArray(texcoord);
        gl::EnableVertexAttribArray(color);

        gl::VertexAttribPointer(position, 2, gl::FLOAT, gl::FALSE, size_of::<UiVertex>() as i32, offset_of!(UiVertex, pos) as *const _);
        gl::VertexAttribPointer(texcoord, 2, gl::FLOAT, gl::FALSE, size_of::<UiVertex>() as i32, offset_of!(UiVertex, uv) as *const _);
        gl::VertexAttribPointer(color, 4, gl::UNSIGNED_BYTE, gl::TRUE, size_of::<UiVertex>() as i32, offset_of!(UiVertex, col) as *const _);
    }
}

impl<'a> UiBatch<'a> {
    pub fn new(max_quads_per_batch: usize, texture: &'a Texture) -> UiBatch<'a> {
        UiBatch {
            vertices: Vec::with_capacity(max_quads_per_batch * 4),
            indices: Vec::with_capacity(max_quads_per_batch * 6),
            texture
        }
    }
}

impl<'a> UiPaintPass<'a> {
    pub fn draw_sprite(self: &mut Self, texture: &'a Texture, position: Vector2, size: Option<Vector2>, pivot: Vector2, rotation: f32, tex_rect: Option<Rectangle>, tint: Color32) {
        let batch = if let Some(b) = self.batch.take() {
            if b.texture != texture || b.vertices.len() >= self.max_quads * 4 {
                self.draw_batch(&b);
                self.batch.insert(UiBatch::new(self.max_quads, texture))
            }
            else {
                self.batch.insert(b)
            }
        }
        else {
            self.batch.insert(UiBatch::new(self.max_quads, texture))
        };

        let size = match size {
            Some(v) => v,
            None => Vector2::new(texture.width() as f32, texture.height() as f32)
        };

        let tex_rect = match tex_rect {
            Some(v) => v,
            None => Rectangle::new(0, 0, texture.width() as i32, texture.height() as i32)
        };

        let offset = pivot * size * -1.0;

        let pos_a = Vector2::new(0.0, 0.0) + offset;
        let pos_b = Vector2::new(size.x, 0.0) + offset;
        let pos_c = Vector2::new(0.0, size.y) + offset;
        let pos_d = Vector2::new(size.x, size.y) + offset;

        let pos_a = position + pos_a.rotate(rotation);
        let pos_b = position + pos_b.rotate(rotation);
        let pos_c = position + pos_c.rotate(rotation);
        let pos_d = position + pos_d.rotate(rotation);

        let uv_scale = 1.0 / Vector2::new(texture.width() as f32, texture.height() as f32);
        let uv_min = Vector2::new(tex_rect.x as f32, tex_rect.y as f32) * uv_scale;
        let uv_max = uv_min + (Vector2::new(tex_rect.w as f32, tex_rect.h as f32) * uv_scale);

        let uv_a = Vector2::new(uv_min.x, uv_min.y);
        let uv_b = Vector2::new(uv_max.x, uv_min.y);
        let uv_c = Vector2::new(uv_min.x, uv_max.y);
        let uv_d = Vector2::new(uv_max.x, uv_max.y);

        let idx_base = batch.vertices.len() as u16;

        batch.vertices.push(UiVertex { pos: pos_a, uv: uv_a, col: tint });
        batch.vertices.push(UiVertex { pos: pos_b, uv: uv_b, col: tint });
        batch.vertices.push(UiVertex { pos: pos_c, uv: uv_c, col: tint });
        batch.vertices.push(UiVertex { pos: pos_d, uv: uv_d, col: tint });

        batch.indices.push(idx_base);
        batch.indices.push(idx_base + 1);
        batch.indices.push(idx_base + 2);

        batch.indices.push(idx_base + 1);
        batch.indices.push(idx_base + 3);
        batch.indices.push(idx_base + 2);
    }

    fn draw_batch(self: &mut Self, batch: &UiBatch) {
        self.vtx_buffer.resize(self.vtx_buffer.size());
        self.idx_buffer.resize(self.idx_buffer.size());

        self.vtx_buffer.set_data(0, &batch.vertices);
        self.idx_buffer.set_data(0, &batch.indices);

        let matrix = Matrix4x4::scale(Vector3::new(2.0 / self.window_size.0 as f32, -2.0 / self.window_size.1 as f32, 1.0))
            * Matrix4x4::translation(Vector3::new(-1.0, 1.0, 0.0));
        
        self.material.apply();
        self.material.shader.set_uniform_mat4("mvp", matrix);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, batch.texture.handle());

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

            self.material.shader.inner.set_uniform_int("mainTexture", 0);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vtx_buffer.handle());
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.idx_buffer.handle());

            setup_vtx_arrays(&self.material.shader);

            gl::DrawElements(gl::TRIANGLES, batch.indices.len() as i32, gl::UNSIGNED_SHORT, 0 as *const _);
        }
    }

    pub fn end(mut self: Self) {
        if let Some(batch) = self.batch.take() {
            self.draw_batch(&batch);
        }
    }
}

impl UiPainter {
    pub fn new(max_quads_per_batch: usize) -> UiPainter {
        UiPainter {
            vtx_buffer: Buffer::new((max_quads_per_batch * 4 * size_of::<UiVertex>()) as isize),
            idx_buffer: Buffer::new((max_quads_per_batch * 6 * size_of::<u16>()) as isize),
            max_quads: max_quads_per_batch,
            material: load_material("content/materials/misc/ui.mat.ron").unwrap(),
        }
    }

    pub fn begin_pass(self: &mut Self, window_size: (u32, u32)) -> UiPaintPass {
        UiPaintPass {
            batch: None,
            material: &self.material,
            window_size,
            vtx_buffer: &mut self.vtx_buffer,
            idx_buffer: &mut self.idx_buffer,
            max_quads: self.max_quads
        }
    }
}