use std::{collections::HashMap, fs::{self, File}, io::{Error, Read}, marker::PhantomData, path::Path, sync::{Arc, RwLock, Weak}};

use basis_universal::{TranscodeParameters, Transcoder, TranscoderTextureFormat};
use gltf::{import_buffers, Gltf};
use lazy_static::lazy_static;
use toml::{map::Map, Table};

use crate::{graphics::{material::{Material, TextureSampler}, model::Model, shader::Shader, texture::{Texture, TextureFormat}}, parse_utils::{parse_vec2, parse_vec3, parse_vec4}};

lazy_static! {
    static ref TEXTURE_CACHE: RwLock<TextureCache> = RwLock::new(TextureCache::new());
    static ref SHADER_CACHE: RwLock<ShaderCache> = RwLock::new(ShaderCache::new());
    static ref MATERIAL_CACHE: RwLock<MaterialCache> = RwLock::new(MaterialCache::new());
    static ref MODEL_CACHE: RwLock<ModelCache> = RwLock::new(ModelCache::new());
}

pub fn load_texture(path: &str) -> Result<Arc<Texture>, ResourceError> {
    let tex_cache = &mut TEXTURE_CACHE.write().unwrap();
    return tex_cache.load(path);
}

pub fn load_shader(path: &str) -> Result<Arc<Shader>, ResourceError> {
    let shader_cache = &mut SHADER_CACHE.write().unwrap();
    return shader_cache.load(path);
}

pub fn load_material(path: &str) -> Result<Arc<Material>, ResourceError> {
    let material_cache = &mut MATERIAL_CACHE.write().unwrap();
    return material_cache.load(path);
}

pub fn load_model(path: &str) -> Result<Arc<Model>, ResourceError> {
    let model_cache = &mut MODEL_CACHE.write().unwrap();
    return model_cache.load(path);
}

#[derive(Debug)]
pub enum ResourceError {
    ParseError,
    IOError(Error)
}

pub trait ResourceLoader<TResource> {
    fn load_resource(path: &str) -> Result<TResource, ResourceError>;
}

pub struct TextureLoader {
}

impl ResourceLoader<Texture> for TextureLoader {
    fn load_resource(path: &str) -> Result<Texture, ResourceError> {    
        let mut tex_file = match File::open(path) {
            Ok(v) => v,
            Err(e) => return Err(ResourceError::IOError(e))
        };

        let mut tex_data = Vec::new();
        tex_file.read_to_end(&mut tex_data).unwrap();

        // create transcoder
        let mut transcoder = Transcoder::new();
        transcoder.prepare_transcoding(&tex_data).unwrap();

        let img_info = transcoder.image_info(&tex_data, 0).unwrap();

        #[cfg(feature = "use_etc1")]
        let (target_fmt, basis_fmt) = (TextureFormat::ETC1, TranscoderTextureFormat::ETC1_RGB);

        #[cfg(not(feature = "use_etc1"))]
        let (target_fmt, basis_fmt) = (TextureFormat::DXT1, TranscoderTextureFormat::BC1_RGB);

        let mut tex = Texture::new(
            target_fmt,
            img_info.m_width as i32,
            img_info.m_height as i32,
            img_info.m_total_levels as i32
        );

        // upload each mip slice
        for tex_level in 0..img_info.m_total_levels {
            // transcode mip level
            let data = transcoder.transcode_image_level(&tex_data, basis_fmt,
                TranscodeParameters {
                    image_index: 0,
                    level_index: tex_level,
                    decode_flags: None,
                    output_row_pitch_in_blocks_or_pixels: None,
                    output_rows_in_pixels: None
                }
            ).unwrap();

            tex.set_texture_data(tex_level as i32, &data);
        }

        Ok(tex)
    }
}

pub struct ShaderLoader {
}

impl ResourceLoader<Shader> for ShaderLoader {
    fn load_resource(path: &str) -> Result<Shader, ResourceError> {
        let shader_str = match fs::read_to_string(path) {
            Ok(v) => v,
            Err(e) => return Err(ResourceError::IOError(e))
        };

        let shader_data = match shader_str.parse::<Table>() {
            Ok(v) => v,
            Err(_) => return Err(ResourceError::ParseError)
        };

        let shader_vtx_src = match shader_data["vs"].as_str() {
            Some(v) => v,
            None => return Err(ResourceError::ParseError)
        };

        let shader_frag_src = match shader_data["ps"].as_str() {
            Some(v) => v,
            None => return Err(ResourceError::ParseError)
        };

        let shader_preamble = "#version 100\nprecision highp float;\n";

        Ok(Shader::new(
            format!("{}\n{}", shader_preamble, shader_vtx_src).as_str(),
            format!("{}\n{}", shader_preamble, shader_frag_src).as_str()
        ))
    }
}

pub struct ModelLoader {
}

impl ResourceLoader<Model> for ModelLoader {
    fn load_resource(path: &str) -> Result<Model, ResourceError> {
        let model_name = Path::new(path).file_stem().unwrap();
        let base_path = Path::new(path).parent().unwrap();

        let material_path = format!("{}/{}", base_path.to_str().unwrap(), model_name.to_str().unwrap());

        let gltf = match Gltf::open(path) {
            Ok(v) => v,
            Err(e) => {
                match e {
                    gltf::Error::Io(error) => {
                        return Err(ResourceError::IOError(error))
                    },
                    _ => {
                        return Err(ResourceError::ParseError)
                    }
                }
            }
        };

        let buffers = match import_buffers(&gltf.document, Some(base_path), gltf.blob) {
            Ok(v) => v,
            Err(e) => {
                match e {
                    gltf::Error::Io(error) => {
                        return Err(ResourceError::IOError(error))
                    },
                    _ => {
                        return Err(ResourceError::ParseError)
                    }
                }
            }
        };

        Ok(Model::from_gltf(&gltf.document, &buffers, &material_path))
    }
}

pub struct MaterialLoader {
}

impl ResourceLoader<Material> for MaterialLoader {
    fn load_resource(path: &str) -> Result<Material, ResourceError> {
        let material_str = match fs::read_to_string(path) {
            Ok(v) => v,
            Err(e) => return Err(ResourceError::IOError(e))
        };

        let material_data = match material_str.parse::<Table>() {
            Ok(v) => v,
            Err(_) => return Err(ResourceError::ParseError)
        };

        let shader_path = match material_data["shader"].as_str() {
            Some(v) => v,
            None => return Err(ResourceError::ParseError)
        };

        let texture = if material_data.contains_key("texture2D") { match material_data["texture2D"].as_table() {
            Some(v) => v,
            None => return Err(ResourceError::ParseError)
        } } else { &Map::new() };

        let float = if material_data.contains_key("float") { match material_data["float"].as_table() {
            Some(v) => v,
            None => return Err(ResourceError::ParseError)
        } } else { &Map::new() };

        let vec2 = if material_data.contains_key("vec2") { match material_data["vec2"].as_table() {
            Some(v) => v,
            None => return Err(ResourceError::ParseError)
        } } else { &Map::new() };

        let vec3 = if material_data.contains_key("vec3") { match material_data["vec3"].as_table() {
            Some(v) => v,
            None => return Err(ResourceError::ParseError)
        } } else { &Map::new() };

        let vec4 = if material_data.contains_key("vec4") { match material_data["vec4"].as_table() {
            Some(v) => v,
            None => return Err(ResourceError::ParseError)
        } } else { &Map::new() };

        let shader = load_shader(shader_path)?;
        let mut material = Material::new(shader);

        if material_data.contains_key("transparent") {
            if let Some(transparent) = material_data["transparent"].as_bool() {
                material.transparent = transparent;
            }
            else {
                return Err(ResourceError::ParseError);
            }
        }

        if material_data.contains_key("enable-cull") {
            if let Some(enable_cull) = material_data["enable-cull"].as_bool() {
                material.enable_cull = enable_cull;
            }
            else {
                return Err(ResourceError::ParseError);
            }
        }

        if material_data.contains_key("cull") {
            match material_data["cull"].as_str() {
                Some("front") => {
                    material.cull = gl::FRONT;
                }
                Some("back") => {
                    material.cull = gl::BACK;
                }
                _ => {
                    return Err(ResourceError::ParseError);
                }
            }
        }

        if material_data.contains_key("depth_test") {
            if let Some(depth_test) = material_data["depth_test"].as_bool() {
                material.depth_test = depth_test;
            }
            else {
                return Err(ResourceError::ParseError);
            }
        }

        if material_data.contains_key("depth_write") {
            if let Some(depth_write) = material_data["depth_write"].as_bool() {
                material.depth_write = depth_write;
            }
            else {
                return Err(ResourceError::ParseError);
            }
        }

        if material_data.contains_key("depth_cmp") {
            match material_data["depth_cmp"].as_str() {
                Some("always") => {
                    material.depth_cmp = gl::ALWAYS;
                }
                Some("never") => {
                    material.depth_cmp = gl::NEVER;
                }
                Some("equal") => {
                    material.depth_cmp = gl::EQUAL;
                }
                Some("not-equal") => {
                    material.depth_cmp = gl::NOTEQUAL;
                }
                Some("less") => {
                    material.depth_cmp = gl::LESS;
                }
                Some("greater") => {
                    material.depth_cmp = gl::GREATER;
                }
                Some("less-or-equal") => {
                    material.depth_cmp = gl::LEQUAL;
                }
                Some("greater-or-equal") => {
                    material.depth_cmp = gl::GEQUAL;
                }
                _ => {
                    return Err(ResourceError::ParseError);
                }
            }
        }

        if material_data.contains_key("blend") {
            if let Some(blend) = material_data["blend"].as_bool() {
                material.blend = blend;
            }
            else {
                return Err(ResourceError::ParseError);
            }
        }

        if material_data.contains_key("blend_op") {
            match material_data["blend_op"].as_str() {
                Some("add") => {
                    material.depth_cmp = gl::FUNC_ADD;
                }
                Some("subtract") => {
                    material.depth_cmp = gl::FUNC_SUBTRACT;
                }
                Some("reverse-subtract") => {
                    material.depth_cmp = gl::FUNC_REVERSE_SUBTRACT;
                }
                _ => {
                    return Err(ResourceError::ParseError);
                }
            }
        }

        let parse_blend_factor = |v| {
            match v {
                Some("zero") => Some(gl::ZERO),
                Some("one") => Some(gl::ONE),
                Some("src-color") => Some(gl::SRC_COLOR),
                Some("src-alpha") => Some(gl::SRC_ALPHA),
                Some("dst-color") => Some(gl::DST_COLOR),
                Some("dst-alpha") => Some(gl::DST_ALPHA),
                Some("inv-src-color") => Some(gl::ONE_MINUS_SRC_COLOR),
                Some("inv-src-alpha") => Some(gl::ONE_MINUS_SRC_ALPHA),
                Some("inv-dst-color") => Some(gl::ONE_MINUS_DST_COLOR),
                Some("inv-dst-alpha") => Some(gl::ONE_MINUS_DST_ALPHA),
                _ => None
            }
        };

        if material_data.contains_key("blend-src") {
            if let Some(blend_src) = parse_blend_factor(material_data["blend-src"].as_str()) {
                material.blend_src = blend_src;
            }
            else {
                return Err(ResourceError::ParseError);
            }
        }

        if material_data.contains_key("blend-dst") {
            if let Some(blend_dst) = parse_blend_factor(material_data["blend-dst"].as_str()) {
                material.blend_dst = blend_dst;
            }
            else {
                return Err(ResourceError::ParseError);
            }
        }

        for p in texture {
            let tbl = p.1.as_table().unwrap();

            let texpath = match tbl["path"].as_str() {
                Some(v) => v,
                None => return Err(ResourceError::ParseError)
            };

            let filter = if tbl.contains_key("filter") {
                match tbl["filter"].as_bool() {
                    Some(v) => v,
                    None => return Err(ResourceError::ParseError)
                }
            }
            else {
                true
            };

            let wrap_s = if tbl.contains_key("wrap_s") {
                match tbl["wrap_s"].as_bool() {
                    Some(v) => v,
                    None => return Err(ResourceError::ParseError)
                }
            }
            else {
                true
            };

            let wrap_t = if tbl.contains_key("wrap_t") {
                match tbl["wrap_t"].as_bool() {
                    Some(v) => v,
                    None => return Err(ResourceError::ParseError)
                }
            }
            else {
                true
            };

            let texture = load_texture(texpath)?;

            material.texture.insert(p.0.to_owned(), TextureSampler { texture, filter, wrap_s, wrap_t });
        }

        for p in float {
            let val = match p.1.as_float() {
                Some(v) => v as f32,
                None => return Err(ResourceError::ParseError)
            };

            material.float.insert(p.0.to_owned(), val);
        }

        for p in vec2 {
            let val_str = match p.1.as_str() {
                Some(v) => v,
                None => return Err(ResourceError::ParseError)
            };

            let val = parse_vec2(val_str);
            material.vec2.insert(p.0.to_owned(), val);
        }

        for p in vec3 {
            let val_str = match p.1.as_str() {
                Some(v) => v,
                None => return Err(ResourceError::ParseError)
            };

            let val = parse_vec3(val_str);
            material.vec3.insert(p.0.to_owned(), val);
        }

        for p in vec4 {
            let val_str = match p.1.as_str() {
                Some(v) => v,
                None => return Err(ResourceError::ParseError)
            };

            let val = parse_vec4(val_str);
            material.vec4.insert(p.0.to_owned(), val);
        }

        Ok(material)
    }
}

/// Implementation of a smart cache with ref counted resources
/// Attempts to load the same resource path more than once will return a reference to the same resource
/// If all references to the resource are dropped, the resource will be unloaded
pub struct ResourceCache<TResource, TResourceLoader>
    where TResourceLoader: ResourceLoader<TResource>
{
    cache: HashMap<String, Weak<TResource>>,
    phantom: PhantomData<TResourceLoader>
}

impl<TResource, TResourceLoader> ResourceCache<TResource, TResourceLoader> 
    where TResourceLoader: ResourceLoader<TResource>
{
    pub fn new() -> ResourceCache<TResource, TResourceLoader> {
        ResourceCache::<TResource, TResourceLoader> {
            cache: HashMap::new(),
            phantom: PhantomData::default()
        }
    }

    pub fn load(self: &mut Self, path: &str) -> Result<Arc<TResource>, ResourceError> {
        if self.cache.contains_key(path) {
            // try and get a reference to the resource, upgraded to a new Rc
            // if that fails, the resource has been unloaded (we'll just load a new one)
            let res = self.cache[path].clone().upgrade();
            match res {
                Some(v) => {
                    return Ok(v);
                }
                None => {
                    self.cache.remove(path);
                }
            };
        }

        println!("Loading {}: {}", std::any::type_name::<TResource>(), path);

        let tex = match TResourceLoader::load_resource(path) {
            Ok(v) => v,
            Err(e) => {
                println!("\t FAILED: {:?}", e);
                return Err(e);
            }
        };

        let res = Arc::new(tex);
        let store = Arc::downgrade(&res.clone());

        self.cache.insert(path.to_owned(), store);
        return Ok(res);
    }
}

pub type TextureCache = ResourceCache<Texture, TextureLoader>;
pub type ShaderCache = ResourceCache<Shader, ShaderLoader>;
pub type MaterialCache = ResourceCache<Material, MaterialLoader>;
pub type ModelCache = ResourceCache<Model, ModelLoader>;