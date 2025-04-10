use std::{collections::HashMap, fs::File, io::Error, marker::PhantomData, sync::{Arc, RwLock, Weak}};

use ktx::KtxInfo;
use lazy_static::lazy_static;

use crate::graphics::{gfx::{GL_COMPRESSED_RGBA_S3TC_DXT1_EXT, GL_COMPRESSED_RGBA_S3TC_DXT3_EXT, GL_COMPRESSED_RGB_S3TC_DXT1_EXT}, texture::{Texture, TextureFormat}};

lazy_static! {
    static ref TEXTURE_CACHE: RwLock<TextureCache> = RwLock::new(TextureCache::new());
}

pub fn load_texture(path: &str) -> Result<Arc<Texture>, ResourceError> {
    let tex_cache = &mut TEXTURE_CACHE.write().unwrap();
    return tex_cache.load(path);
}

pub fn load_env(env_name: &str) -> [Arc<Texture>;6] {
    let env_ft = load_texture(format!("/cd/content/env/{}1ft.ktx", env_name).as_str()).unwrap();
    let env_bk = load_texture(format!("/cd/content/env/{}1bk.ktx", env_name).as_str()).unwrap();
    let env_lf = load_texture(format!("/cd/content/env/{}1lf.ktx", env_name).as_str()).unwrap();
    let env_rt = load_texture(format!("/cd/content/env/{}1rt.ktx", env_name).as_str()).unwrap();
    let env_up = load_texture(format!("/cd/content/env/{}1up.ktx", env_name).as_str()).unwrap();
    let env_dn = load_texture(format!("/cd/content/env/{}1dn.ktx", env_name).as_str()).unwrap();

    [env_ft, env_bk, env_lf, env_rt, env_up, env_dn]
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
        let tex_file = match File::open(path) {
            Ok(v) => v,
            Err(e) => return Err(ResourceError::IOError(e))
        };

        // decode KTX texture
        let decoder = match ktx::Decoder::new(tex_file) {
            Ok(v) => v,
            Err(_) => return Err(ResourceError::ParseError)
        };

        // TODO: I think eventually I'd like to switch to Basis Universal for textures

        // find appropriate texture format
        let tex_fmt = if decoder.gl_type() == gl::UNSIGNED_BYTE && decoder.gl_format() == gl::RGBA {
            TextureFormat::RGBA8888
        } else if decoder.gl_type() == gl::UNSIGNED_SHORT_5_6_5 && decoder.gl_format() == gl::RGB {
            TextureFormat::RGB565
        } else if decoder.gl_type() == gl::UNSIGNED_SHORT_4_4_4_4 && decoder.gl_format() == gl::RGBA {
            TextureFormat::RGBA4444
        } else if decoder.gl_internal_format() == GL_COMPRESSED_RGB_S3TC_DXT1_EXT {
            TextureFormat::DXT1
        } else if decoder.gl_internal_format() == GL_COMPRESSED_RGBA_S3TC_DXT1_EXT {
            TextureFormat::DXT1A
        }  else if decoder.gl_internal_format() == GL_COMPRESSED_RGBA_S3TC_DXT3_EXT {
            TextureFormat::DXT3
        } else {
            println!("Failed decoding KTX image: unsupported pixel format");
            return Err(ResourceError::ParseError);
        };

        let mut tex = Texture::new(
            tex_fmt,
            decoder.pixel_width() as i32,
            decoder.pixel_height() as i32,
            decoder.mipmap_levels() as i32);

        // upload each mip slice
        let mut level: i32 = 0;
        for tex_level in decoder.read_textures() {
            tex.set_texture_data(level, &tex_level);
            level += 1;
        }

        Ok(tex)
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