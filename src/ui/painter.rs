use crate::{asset_loader::{load_material, MaterialHandle}, graphics::{buffer::Buffer, texture::Texture}, math::{Matrix4x4, Vector2, Vector3}, misc::{Color32, Rectangle}};

use super::ui_vertex::UiVertex;

pub struct UiPainter {
    texture: Option<u32>,
    max_quads: usize,
    material: MaterialHandle,
    vertices: Vec<UiVertex>,
    indices: Vec<u16>,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    window_size: (u32, u32),
}

impl UiPainter {
    pub fn new(max_quads: usize) -> UiPainter {
        UiPainter {
            texture: None,
            max_quads,
            material: load_material("content/materials/misc/ui.mat.ron").unwrap(),
            vertices: Vec::with_capacity(max_quads * 4),
            indices: Vec::with_capacity(max_quads * 6),
            vertex_buffer: Buffer::new((max_quads * 4 * size_of::<UiVertex>()) as isize),
            index_buffer: Buffer::new((max_quads * 6 * size_of::<u16>()) as isize),
            window_size: (0, 0),
        }
    }

    fn flush_batch(self: &mut Self) {
        if self.vertices.len() == 0 {
            return;
        }

        let tex = self.texture.as_ref().unwrap();

        self.vertex_buffer.resize(self.vertex_buffer.size());
        self.index_buffer.resize(self.index_buffer.size());

        self.vertex_buffer.set_data(0, &self.vertices);
        self.index_buffer.set_data(0, &self.indices);

        let matrix = Matrix4x4::scale(Vector3::new(2.0 / self.window_size.0 as f32, -2.0 / self.window_size.1 as f32, 1.0))
            * Matrix4x4::translation(Vector3::new(-1.0, 1.0, 0.0));
        
        self.material.apply();
        self.material.shader.set_uniform_mat4("mvp", matrix);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, *tex);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

            self.material.shader.inner.set_uniform_int("mainTexture", 0);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer.handle());
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer.handle());

            UiVertex::setup_vtx_arrays(&self.material.shader);

            gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_SHORT, 0 as *const _);
        }

        self.vertices.clear();
        self.indices.clear();
        self.texture = None;
    }

    pub fn begin(&mut self, window_size: (u32, u32)) {
        self.window_size = window_size;
    }

    pub fn end(&mut self) {
        self.flush_batch();
    }

    pub fn draw_sprite(&mut self, tex: &Texture, position: Vector2, size: Vector2, pivot: Vector2, rotation: f32, tex_rect: Rectangle, tint: Color32) {
        if let Some(prev_tex) = &self.texture {
            if *prev_tex != tex.handle() {
                // flush batch
                self.flush_batch();
            }
        }

        if self.vertices.len() >= self.max_quads * 4 {
            self.flush_batch();
        }

        self.texture = Some(tex.handle());

        let offset = pivot * -1.0;

        let pos_a = Vector2::new(0.0, 0.0) + offset;
        let pos_b = Vector2::new(size.x, 0.0) + offset;
        let pos_c = Vector2::new(0.0, size.y) + offset;
        let pos_d = Vector2::new(size.x, size.y) + offset;

        let pos_a = position + pos_a.rotate(rotation);
        let pos_b = position + pos_b.rotate(rotation);
        let pos_c = position + pos_c.rotate(rotation);
        let pos_d = position + pos_d.rotate(rotation);

        let uv_scale = 1.0 / Vector2::new(tex.width() as f32, tex.height() as f32);
        let uv_min = Vector2::new(tex_rect.x as f32, tex_rect.y as f32) * uv_scale;
        let uv_max = uv_min + (Vector2::new(tex_rect.w as f32, tex_rect.h as f32) * uv_scale);

        let uv_a = Vector2::new(uv_min.x, uv_min.y);
        let uv_b = Vector2::new(uv_max.x, uv_min.y);
        let uv_c = Vector2::new(uv_min.x, uv_max.y);
        let uv_d = Vector2::new(uv_max.x, uv_max.y);

        let idx_base = self.vertices.len() as u16;

        self.vertices.push(UiVertex { pos: pos_a, uv: uv_a, col: tint });
        self.vertices.push(UiVertex { pos: pos_b, uv: uv_b, col: tint });
        self.vertices.push(UiVertex { pos: pos_c, uv: uv_c, col: tint });
        self.vertices.push(UiVertex { pos: pos_d, uv: uv_d, col: tint });

        self.indices.push(idx_base);
        self.indices.push(idx_base + 1);
        self.indices.push(idx_base + 2);

        self.indices.push(idx_base + 1);
        self.indices.push(idx_base + 3);
        self.indices.push(idx_base + 2);
    }

    pub fn draw_nineslice(&mut self, texture: &Texture,
        position: Vector2,
        size: Vector2,
        pivot: Vector2,
        rotation: f32,
        tex_rect: Option<Rectangle>,
        borders: (i32, i32, i32, i32),
        color: Color32)
    {
        let (l, r, t, b) = borders;
        let lr = l + r;
        let tb = t + b;
        let (tx, ty, tw, th) = match tex_rect {
            Some(v) => (v.x, v.y, v.w, v.h),
            None => (0, 0, texture.width(), texture.height())
        };
        let w = size.x as i32;
        let h = size.y as i32;

        // top left
        let tex_rect = Rectangle::new(tx, ty, l, t);
        self.draw_sprite(&texture,
            position,
            Vector2::new(l as f32, t as f32),
            pivot,
            rotation,
            tex_rect,
            color);

        // top middle
        let tex_rect = Rectangle::new(tx + l, ty, tw - lr, t);
        self.draw_sprite(&texture,
            position,
            Vector2::new((w - lr) as f32, t as f32),
            pivot - Vector2::new(l as f32, 0.0),
            rotation,
            tex_rect,
            color);

        // top right
        let tex_rect = Rectangle::new(tx + (tw - r), ty, r, t);
        self.draw_sprite(&texture,
            position,
            Vector2::new(r as f32, t as f32),
            pivot - Vector2::new((w - r) as f32, 0.0),
            rotation,
            tex_rect,
            color);

        // middle left
        let tex_rect = Rectangle::new(tx, ty + t, l, th - tb);
        self.draw_sprite(&texture,
            position,
            Vector2::new(l as f32, (h - tb) as f32),
            pivot - Vector2::new(0.0, t as f32),
            rotation,
            tex_rect,
            color);

        // middle
        let tex_rect = Rectangle::new(tx + l, ty + t, tw - lr, th - tb);
        self.draw_sprite(&texture,
            position,
            Vector2::new((w - lr) as f32, (h - tb) as f32),
            pivot - Vector2::new(l as f32, t as f32),
            rotation,
            tex_rect,
            color);

        // middle right
        let tex_rect = Rectangle::new(tx + (tw - r), ty + t, r, th - tb);
        self.draw_sprite(&texture,
            position,
            Vector2::new(r as f32, (h - tb) as f32),
            pivot - Vector2::new((w - r) as f32, t as f32),
            rotation,
            tex_rect,
            color);

        // bottom left
        let tex_rect = Rectangle::new(tx, ty + (th - b), l, b);
        self.draw_sprite(&texture,
            position,
            Vector2::new(l as f32, b as f32),
            pivot - Vector2::new(0.0, (h - b) as f32),
            rotation,
            tex_rect,
            color);

        // bottom middle
        let tex_rect = Rectangle::new(tx + l, ty + (th - b), tw - lr, b);
        self.draw_sprite(&texture,
            position,
            Vector2::new((w - lr) as f32, b as f32),
            pivot - Vector2::new(l as f32, (h - b) as f32),
            rotation,
            tex_rect,
            color);

        // bottom right
        let tex_rect = Rectangle::new(tx + (tw - r), ty + (th - b), r, b);
        self.draw_sprite(&texture,
            position,
            Vector2::new(r as f32, b as f32),
            pivot - Vector2::new((w - r) as f32, (h - b) as f32),
            rotation,
            tex_rect,
            color);
    }
}