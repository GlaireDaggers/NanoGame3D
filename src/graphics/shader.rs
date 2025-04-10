use gamemath::Mat4;

use crate::misc::{Vector2, Vector3, Vector4};

use super::gfx::{create_program, get_attrib_location, set_uniform_float, set_uniform_int, set_uniform_mat4, set_uniform_vec2, set_uniform_vec3, set_uniform_vec4};

pub struct Shader {
    handle: u32
}

impl Shader {
    pub fn new(vtx_source: &str, frag_source: &str) -> Shader {
        Shader { handle: create_program(vtx_source, frag_source) }
    }

    pub fn set_active(self: &Shader) {
        unsafe {
            gl::UseProgram(self.handle);
        }
    }

    pub fn get_attribute_location(self: &Shader, name: &str) -> u32 {
        get_attrib_location(self.handle, name) as u32
    }

    pub fn set_uniform_int(self: &Shader, name: &str, value: i32) {
        set_uniform_int(self.handle, name, value);
    }

    pub fn set_uniform_float(self: &Shader, name: &str, value: f32) {
        set_uniform_float(self.handle, name, value);
    }

    pub fn set_uniform_vec2(self: &Shader, name: &str, value: Vector2) {
        set_uniform_vec2(self.handle, name, value);
    }

    pub fn set_uniform_vec3(self: &Shader, name: &str, value: Vector3) {
        set_uniform_vec3(self.handle, name, value);
    }

    pub fn set_uniform_vec4(self: &Shader, name: &str, value: Vector4) {
        set_uniform_vec4(self.handle, name, value);
    }

    pub fn set_uniform_mat4(self: &Shader, name: &str, value: Mat4) {
        set_uniform_mat4(self.handle, name, value);
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.handle);
        }
    }
}