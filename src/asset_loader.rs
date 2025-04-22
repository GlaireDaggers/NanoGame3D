use std::{collections::HashMap, fs::{self, File}, io::{Error, Read}, marker::PhantomData, path::Path, sync::{Arc, RwLock, Weak}};

use basis_universal::{TranscodeParameters, Transcoder, TranscoderTextureFormat};
use gltf::{import_buffers, Gltf};
use lazy_static::lazy_static;
use qoi::decode_to_vec;
use toml::Table;

use crate::{effect::effect_data::EffectData, graphics::{material::Material, model::Model, shader::Shader, texture::{Texture, TextureFormat}}, misc::Color32};

lazy_static! {
    static ref TEXTURE_CACHE: RwLock<TextureCache> = RwLock::new(TextureCache::new());
    static ref SHADER_CACHE: RwLock<ShaderCache> = RwLock::new(ShaderCache::new());
    static ref MATERIAL_CACHE: RwLock<MaterialCache> = RwLock::new(MaterialCache::new());
    static ref MODEL_CACHE: RwLock<ModelCache> = RwLock::new(ModelCache::new());
    static ref EFFECT_CACHE: RwLock<EffectCache> = RwLock::new(EffectCache::new());
}

pub fn unload_texture(path: &str) {
    let tex_cache = &mut TEXTURE_CACHE.write().unwrap();
    tex_cache.unload(path);
}

pub fn load_texture(path: &str) -> Result<Arc<Texture>, ResourceError> {
    let tex_cache = &mut TEXTURE_CACHE.write().unwrap();
    return tex_cache.load(path);
}

pub fn unload_shader(path: &str) {
    let shader_cache = &mut SHADER_CACHE.write().unwrap();
    shader_cache.unload(path);
}

pub fn load_shader(path: &str) -> Result<Arc<Shader>, ResourceError> {
    let shader_cache = &mut SHADER_CACHE.write().unwrap();
    return shader_cache.load(path);
}

pub fn unload_material(path: &str) {
    let material_cache = &mut MATERIAL_CACHE.write().unwrap();
    material_cache.unload(path);
}

pub fn load_material(path: &str) -> Result<Arc<Material>, ResourceError> {
    let material_cache = &mut MATERIAL_CACHE.write().unwrap();
    return material_cache.load(path);
}

pub fn unload_model(path: &str) {
    let model_cache = &mut MODEL_CACHE.write().unwrap();
    model_cache.unload(path);
}

pub fn load_model(path: &str) -> Result<Arc<Model>, ResourceError> {
    let model_cache = &mut MODEL_CACHE.write().unwrap();
    return model_cache.load(path);
}

pub fn unload_effect(path: &str) {
    let effect_cache = &mut EFFECT_CACHE.write().unwrap();
    effect_cache.unload(path);
}

pub fn load_effect(path: &str) -> Result<Arc<EffectData>, ResourceError> {
    let effect_cache = &mut EFFECT_CACHE.write().unwrap();
    return effect_cache.load(path);
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

        if path.ends_with(".basis") {
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
        else if path.ends_with(".qoi") {
            let (header, decoded) = match decode_to_vec(tex_data) {
                Ok(v) => v,
                Err(e) => {
                    println!("Failed decoding QOI image: {:?}", e);
                    return Err(ResourceError::ParseError);
                },
            };

            let pixels = match header.channels {
                qoi::Channels::Rgb => {
                    decoded.chunks_exact(3).map(|x| Color32::new(x[0], x[1], x[2], 255)).collect::<Vec<_>>()
                },
                qoi::Channels::Rgba => {
                    decoded.chunks_exact(4).map(|x| Color32::new(x[0], x[1], x[2], x[3])).collect::<Vec<_>>()
                }
            };

            let mut tex = Texture::new(
                TextureFormat::RGBA8888,
                header.width as i32,
                header.height as i32,
                1
            );

            tex.set_texture_data(0, &pixels);

            Ok(tex)
        }
        else {
            println!("Unsupported texture format");
            Err(ResourceError::ParseError)
        }
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

pub struct EffectLoader {
}

impl ResourceLoader<EffectData> for EffectLoader {
    fn load_resource(path: &str) -> Result<EffectData, ResourceError> {
        let effect_str = match fs::read_to_string(path) {
            Ok(v) => v,
            Err(e) => return Err(ResourceError::IOError(e))
        };

        let effect_data = match ron::from_str::<EffectData>(&effect_str) {
            Ok(v) => v,
            Err(e) => {
                println!("PARSE ERROR: {:?}", e);
                return Err(ResourceError::ParseError);
            }
        };

        Ok(effect_data)
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

        let material_data = match ron::from_str::<Material>(&material_str) {
            Ok(v) => v,
            Err(e) => {
                println!("PARSE ERROR: {:?}", e);
                return Err(ResourceError::ParseError);
            }
        };

        Ok(material_data)
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

    pub fn unload(self: &mut Self, path: &str) {
        self.cache.remove(path);
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
pub type EffectCache = ResourceCache<EffectData, EffectLoader>;