use std::{ffi::CStr, fs::File};

use bsp::bspfile::BspFile;
use graphics::gfx::{create_program, set_uniform_float};

extern crate sdl2;
extern crate gl;
extern crate byteorder;

pub mod bsp;
pub mod misc;
pub mod graphics;
pub mod asset_loader;

const VERTEX_SHADER_SOURCE: &str = r#"#version 100
attribute vec2 in_position;
varying vec2 position;

void main() {
    position = in_position;
    gl_Position = vec4(in_position - 0.5, 0.0, 1.0);
}"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"#version 100
varying mediump vec2 position;
uniform mediump float blue;

void main() {
    gl_FragColor = vec4(position, blue, 1.0);
}"#;

fn main() {
    let sdl = sdl2::init().unwrap();
    let sdl_video = sdl.video().unwrap();

    #[cfg(feature = "gles2")]
    {
        let gl_attr = sdl_video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::GLES);
        gl_attr.set_context_version(2, 0);
    }

    let window = sdl_video
        .window("NanoGame3D", 1280, 720)
        .opengl()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    gl::load_with(|s| sdl_video.gl_get_proc_address(s) as *const _);

    let gl_ver = unsafe { CStr::from_ptr(gl::GetString(gl::VERSION) as *const i8) }.to_str().unwrap();
    let gl_renderer = unsafe { CStr::from_ptr(gl::GetString(gl::RENDERER) as *const i8) }.to_str().unwrap();
    println!("{} (GL: {})", gl_renderer, gl_ver);

    let program = create_program(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
    unsafe { gl::UseProgram(program); }

    set_uniform_float(program, "blue", 0.8);

    unsafe { gl::ClearColor(0.25, 0.5, 1.0, 1.0) };

    // load map data
    let mut bsp_stream = File::open("content/maps/demo1.bsp").expect("Failed opening BSP file");
    let bsp_data = BspFile::new(&mut bsp_stream);

    println!("BSP LOADED\nBrushes: {}", bsp_data.brush_lump.brushes.len());

    // test: load texture
    let tex = asset_loader::load_texture("content/textures/e1u1/metal1_1.ktx").expect("Failed loading texture");

    println!("TEXTURE LOADED (width: {}, height: {}, format: {:?})", tex.width(), tex.height(), tex.format());

    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => break 'main,
                _ => {},
            }
        }

        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT) };
        window.gl_swap_window();
    }
}
