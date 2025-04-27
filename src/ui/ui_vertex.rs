use std::mem::offset_of;

use crate::{asset_loader::ShaderHandle, math::Vector2, misc::Color32};

pub struct UiVertex {
    pub pos: Vector2,
    pub uv: Vector2,
    pub col: Color32,
}

impl UiVertex {
    pub fn setup_vtx_arrays(shader: &ShaderHandle) {
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
}