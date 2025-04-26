use std::collections::HashMap;

use serde::Deserialize;

use crate::{asset_loader::{ShaderHandle, TextureHandle}, math::{Vector2, Vector3, Vector4}, serialization::SerializedResource};

#[derive(Deserialize, Clone)]
pub struct TextureSampler {
    pub texture: SerializedResource<TextureHandle>,
    pub filter: bool,
    pub wrap_s: bool,
    pub wrap_t: bool
}

#[derive(Deserialize, Clone)]
pub enum CullMode {
    Off,
    Front,
    Back
}

impl Default for CullMode {
    fn default() -> Self {
        CullMode::Back
    }
}

#[derive(Deserialize, Clone)]
pub enum DepthCompare {
    Always,
    Never,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessOrEqual,
    GreaterOrEqual,
}

impl Default for DepthCompare {
    fn default() -> Self {
        DepthCompare::LessOrEqual
    }
}

#[derive(Deserialize, Clone)]
pub enum BlendEquation {
    Add,
    Subtract,
    ReverseSubtract,
}

impl Default for BlendEquation {
    fn default() -> Self {
        BlendEquation::Add
    }
}

#[derive(Deserialize, Clone)]
pub enum BlendFunction {
    Zero,
    One,
    SrcColor,
    SrcAlpha,
    DstColor,
    DstAlpha,
    OneMinusSrcColor,
    OneMinusSrcAlpha,
    OneMinusDstColor,
    OneMinusDstAlpha,
}

impl BlendFunction {
    pub fn to_gl(&self) -> gl::types::GLenum {
        match self {
            BlendFunction::Zero => gl::ZERO,
            BlendFunction::One => gl::ONE,
            BlendFunction::SrcColor => gl::SRC_COLOR,
            BlendFunction::SrcAlpha => gl::SRC_ALPHA,
            BlendFunction::DstColor => gl::DST_COLOR,
            BlendFunction::DstAlpha => gl::DST_ALPHA,
            BlendFunction::OneMinusSrcColor => gl::ONE_MINUS_SRC_COLOR,
            BlendFunction::OneMinusSrcAlpha => gl::ONE_MINUS_SRC_ALPHA,
            BlendFunction::OneMinusDstColor => gl::ONE_MINUS_DST_COLOR,
            BlendFunction::OneMinusDstAlpha => gl::ONE_MINUS_DST_ALPHA,
        }
    }
}

fn blend_func_default_src() -> BlendFunction {
    BlendFunction::One
}

fn blend_func_default_dst() -> BlendFunction {
    BlendFunction::Zero
}

fn default_depth_test() -> bool {
    true
}

fn default_depth_write() -> bool {
    true
}

#[derive(Deserialize, Clone)]
pub enum MaterialParam {
    Float(f32),
    Vec2(Vector2),
    Vec3(Vector3),
    Vec4(Vector4),
    Texture(TextureSampler),
}

#[derive(Deserialize, Clone)]
pub struct Material {
    pub shader : SerializedResource<ShaderHandle>,
    
    #[serde(default)]
    pub transparent : bool,
    
    #[serde(default)]
    pub cull: CullMode,

    #[serde(default = "default_depth_test")]
    pub depth_test : bool,

    #[serde(default = "default_depth_write")]
    pub depth_write : bool,

    #[serde(default)]
    pub depth_cmp : DepthCompare,

    #[serde(default)]
    pub blend : bool,

    #[serde(default)]
    pub blend_equation : BlendEquation,

    #[serde(default = "blend_func_default_src")]
    pub blend_src : BlendFunction,

    #[serde(default = "blend_func_default_dst")]
    pub blend_dst : BlendFunction,

    pub params : HashMap<String, MaterialParam>,
}

impl Material {
    pub fn new(shader: ShaderHandle) -> Material {
        Material {
            shader: SerializedResource { inner: shader },
            
            transparent: false,
            cull: CullMode::Back,
            depth_test: true,
            depth_write: true,
            depth_cmp: DepthCompare::LessOrEqual,
            blend: false,
            blend_equation: BlendEquation::Add,
            blend_src: BlendFunction::One,
            blend_dst: BlendFunction::Zero,

            params: HashMap::new(),
        }
    }

    pub fn apply(self: &Self) {
        self.shader.inner.set_active();

        unsafe {
            match self.cull {
                CullMode::Off => {
                    gl::Disable(gl::CULL_FACE);
                }
                CullMode::Back => {
                    gl::Enable(gl::CULL_FACE);
                    gl::CullFace(gl::BACK);
                }
                CullMode::Front => {
                    gl::Enable(gl::CULL_FACE);
                    gl::CullFace(gl::FRONT);
                }
            }
            
            if self.depth_test { gl::Enable(gl::DEPTH_TEST) } else { gl::Disable(gl::DEPTH_TEST) };
            gl::DepthMask(if self.depth_write { gl::TRUE } else { gl::FALSE });

            gl::DepthFunc(match self.depth_cmp {
                DepthCompare::Always => gl::ALWAYS,
                DepthCompare::Never => gl::NEVER,
                DepthCompare::Equal => gl::EQUAL,
                DepthCompare::NotEqual => gl::NOTEQUAL,
                DepthCompare::Less => gl::LESS,
                DepthCompare::Greater => gl::GREATER,
                DepthCompare::LessOrEqual => gl::LEQUAL,
                DepthCompare::GreaterOrEqual => gl::GEQUAL,
            });
            
            if self.blend { gl::Enable(gl::BLEND) } else { gl::Disable(gl::BLEND) };
            gl::BlendEquation(match self.blend_equation {
                BlendEquation::Add => gl::FUNC_ADD,
                BlendEquation::Subtract => gl::FUNC_SUBTRACT,
                BlendEquation::ReverseSubtract => gl::FUNC_REVERSE_SUBTRACT,
            });

            gl::BlendFunc(self.blend_src.to_gl(), self.blend_dst.to_gl());
        }

        let mut cur_tex_slot = 0;

        for param in &self.params {
            match param.1 {
                MaterialParam::Float(val) => {
                    self.shader.inner.set_uniform_float(param.0, *val);
                },
                MaterialParam::Vec2(val) => {
                    self.shader.inner.set_uniform_vec2(param.0, *val);
                },
                MaterialParam::Vec3(val) => {
                    self.shader.inner.set_uniform_vec3(param.0, *val);
                },
                MaterialParam::Vec4(val) => {
                    self.shader.inner.set_uniform_vec4(param.0, *val);
                },
                MaterialParam::Texture(val) => {
                    unsafe {
                        gl::ActiveTexture(gl::TEXTURE0 + cur_tex_slot);
        
                        gl::BindTexture(gl::TEXTURE_2D, val.texture.inner.handle());
        
                        if val.filter {
                            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                        }
                        else {
                            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                        }
        
                        if val.wrap_s {
                            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
                        }
                        else {
                            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32)
                        }
        
                        if val.wrap_t {
                            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
                        }
                        else {
                            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32)
                        }
                    }
        
                    self.shader.inner.set_uniform_int(param.0, cur_tex_slot as i32);
                    cur_tex_slot += 1;
                },
            }
        }
    }
}