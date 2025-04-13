use std::{collections::HashMap, sync::Arc};

use gl::types;

use crate::math::{Matrix4x4, Vector2, Vector3, Vector4};

use super::{shader::Shader, texture::Texture};

#[derive(Clone)]
pub struct TextureSampler {
    pub texture: Arc<Texture>,
    pub filter: bool,
    pub wrap_s: bool,
    pub wrap_t: bool
}

#[derive(Clone)]
pub struct Material {
    pub shader : Arc<Shader>,
    
    pub transparent : bool,
    
    pub cull : types::GLenum,
    pub depth_test : bool,
    pub depth_write : bool,
    pub depth_cmp : types::GLenum,
    pub blend : bool,
    pub blend_equation : types::GLenum,
    pub blend_src : types::GLenum,
    pub blend_dst : types::GLenum,

    pub texture : HashMap<String, TextureSampler>,
    pub float : HashMap<String, f32>,
    pub vec2 : HashMap<String, Vector2>,
    pub vec3 : HashMap<String, Vector3>,
    pub vec4 : HashMap<String, Vector4>,
    pub mat4 : HashMap<String, Matrix4x4>,
}

impl Material {
    pub fn new(shader: Arc<Shader>) -> Material {
        Material {
            shader,
            
            transparent: false,
            cull: gl::BACK,
            depth_test: true,
            depth_write: true,
            depth_cmp: gl::LEQUAL,
            blend: false,
            blend_equation: gl::FUNC_ADD,
            blend_src: gl::ONE,
            blend_dst: gl::ZERO,

            texture: HashMap::new(),
            float: HashMap::new(),
            vec2: HashMap::new(),
            vec3: HashMap::new(),
            vec4: HashMap::new(),
            mat4: HashMap::new(),
        }
    }

    pub fn apply(self: &Self) {
        self.shader.set_active();

        unsafe {
            gl::CullFace(self.cull);
            
            if self.depth_test { gl::Enable(gl::DEPTH_TEST) } else { gl::Disable(gl::DEPTH_TEST) };
            gl::DepthMask(if self.depth_write { gl::TRUE } else { gl::FALSE });
            gl::DepthFunc(self.depth_cmp);
            
            if self.blend { gl::Enable(gl::BLEND) } else { gl::Disable(gl::BLEND) };
            gl::BlendEquation(self.blend_equation);
            gl::BlendFunc(self.blend_src, self.blend_dst);
        }

        let mut cur_tex_slot = 0;
        for p in &self.texture {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + cur_tex_slot);

                gl::BindTexture(gl::TEXTURE_2D, p.1.texture.handle());

                if p.1.filter {
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                }
                else {
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                }

                if p.1.wrap_s {
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
                }
                else {
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32)
                }

                if p.1.wrap_t {
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
                }
                else {
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32)
                }
            }

            self.shader.set_uniform_int(p.0, cur_tex_slot as i32);
            cur_tex_slot += 1;
        }

        for p in &self.float {
            self.shader.set_uniform_float(p.0, *p.1);
        }

        for p in &self.vec2 {
            self.shader.set_uniform_vec2(p.0, *p.1);
        }

        for p in &self.vec3 {
            self.shader.set_uniform_vec3(p.0, *p.1);
        }

        for p in &self.vec4 {
            self.shader.set_uniform_vec4(p.0, *p.1);
        }

        for p in &self.mat4 {
            self.shader.set_uniform_mat4(p.0, *p.1);
        }
    }
}