use std::ptr::null;

use crate::gl_checked;

use super::gfx::{GL_COMPRESSED_RGBA_S3TC_DXT1_EXT, GL_COMPRESSED_RGBA_S3TC_DXT3_EXT, GL_COMPRESSED_RGB_S3TC_DXT1_EXT};

#[derive(Clone, Copy, Debug)]
pub enum TextureFormat {
    RGB565,
    RGBA4444,
    RGBA8888,
    DXT1,
    DXT1A,
    DXT3,
}

pub struct Texture {
    fmt: TextureFormat,
    w: i32,
    h: i32,
    levels: i32,
    handle: u32,
    gl_internal_fmt: u32,
    gl_fmt: u32,
    gl_type: u32,
    is_compressed: bool,
}

impl Texture {
    pub fn new(fmt: TextureFormat, w: i32, h: i32, levels: i32) -> Texture {
        let mut handle = 0;
        unsafe {
            gl::GenTextures(1, &mut handle);
            if handle == 0 {
                panic!("Failed to create GL texture");
            }
        };

        let (gl_internal_fmt, gl_fmt, gl_type, is_compressed) = match fmt {
            TextureFormat::RGB565 => (gl::RGB, gl::RGB, gl::UNSIGNED_SHORT_5_6_5, false),
            TextureFormat::RGBA4444 => (gl::RGBA, gl::RGBA, gl::UNSIGNED_SHORT_4_4_4_4, false),
            TextureFormat::RGBA8888 => (gl::RGBA, gl::RGBA, gl::UNSIGNED_BYTE, false),
            TextureFormat::DXT1 => (GL_COMPRESSED_RGB_S3TC_DXT1_EXT, 0, 0, true),
            TextureFormat::DXT1A => (GL_COMPRESSED_RGBA_S3TC_DXT1_EXT, 0, 0, true),
            TextureFormat::DXT3 => (GL_COMPRESSED_RGBA_S3TC_DXT3_EXT, 0, 0, true),
        };

        unsafe {
            if !is_compressed {
                gl::BindTexture(gl::TEXTURE_2D, handle);

                for i in 0..levels {
                    gl_checked!{ gl::TexImage2D(gl::TEXTURE_2D, i, gl_internal_fmt as i32, w >> i, h >> i, 0, gl_fmt, gl_type, null()) }
                }
            }
        }

        Texture {
            fmt, w, h, levels, handle, gl_internal_fmt, gl_fmt, gl_type, is_compressed
        }
    }

    pub fn set_texture_data<T>(self: &mut Self, level: i32, data: &[T]) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.handle);

            if self.is_compressed {
                let data_size = data.len() * size_of::<T>();
                gl_checked!{ gl::CompressedTexImage2D(gl::TEXTURE_2D, level, self.gl_internal_fmt, self.w >> level, self.h >> level, 0, data_size as i32, data.as_ptr() as *const _) }
            }
            else {
                gl_checked!{ gl::TexSubImage2D(gl::TEXTURE_2D, level, 0, 0, self.w >> level, self.h >> level, self.gl_fmt, self.gl_type, data.as_ptr() as *const _) }
            }
        }
    }

    pub fn set_texture_data_region<T>(self: &mut Self, level: i32, x: i32, y: i32, w: i32, h: i32, data: &[T]) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.handle);

            if self.is_compressed {
                panic!("Can't set region of compressed texture")
            }
            else {
                gl_checked!{ gl::TexSubImage2D(gl::TEXTURE_2D, level, x, y, w, h, self.gl_fmt, self.gl_type, data.as_ptr() as *const _) }
            }
        }
    }

    pub fn width(self: &Self) -> i32 {
        self.w
    }

    pub fn height(self: &Self) -> i32 {
        self.w
    }
    
    pub fn levels(self: &Self) -> i32 {
        self.levels
    }
    
    pub fn format(self: &Self) -> TextureFormat {
        self.fmt
    }

    pub fn handle(self: &Self) -> u32 {
        self.handle
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.handle);
        }
    }
}