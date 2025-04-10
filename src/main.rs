use std::{ffi::CStr, fs::File};

use bsp::{bspfile::BspFile, bsprenderer::{BspMapRenderer, BspMapTextures, NUM_CUSTOM_LIGHT_LAYERS}};
use gamemath::Mat4;
use graphics::gfx::{create_program, set_uniform_float};
use misc::{mat4_translation, Vector3, Vector4, VEC3_UNIT_X, VEC3_UNIT_Y, VEC3_UNIT_Z, VEC3_ZERO};

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

    #[cfg(not(feature = "gles2"))]
    {
        let gl_attr = sdl_video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Compatibility);
        gl_attr.set_context_version(3, 2);
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

    unsafe { gl::ClearColor(0.25, 0.5, 1.0, 1.0); gl::ClearDepthf(1.0) };

    // load map data
    let mut bsp_stream = File::open("content/maps/demo1.bsp").expect("Failed opening BSP file");
    let bsp_data = BspFile::new(&mut bsp_stream);
    println!("BSP MAP LOADED");

    // load map textures
    let bsp_textures = BspMapTextures::new(&bsp_data);
    println!("BSP TEXTURES LOADED");

    // create map renderer
    let mut bsp_renderer = BspMapRenderer::new(&bsp_data);
    println!("BSP RENDERER INITIALIZED");

    let light_layers = [0.0; NUM_CUSTOM_LIGHT_LAYERS];
    bsp_renderer.update(0.0, &light_layers, &bsp_data, &bsp_textures, VEC3_ZERO);

    let mut rot: f32 = 0.0;

    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => break 'main,
                _ => {},
            }
        }

        let win_size = window.size();
        let aspect = win_size.0 as f32 / win_size.1 as f32;

        rot += 0.04;

        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT) };

        let cam_view = Mat4::identity()
            * mat4_translation(Vector3::new(0.0, 0.0, -100.0))
            * Mat4::rotation(rot.to_radians(), VEC3_UNIT_Z);
        let cam_proj = Mat4::perspective(120.0_f32.to_radians(), aspect, 10.0, 10000.0);
        bsp_renderer.draw_opaque(&bsp_data, &bsp_textures, 0.0, cam_view, cam_proj);

        window.gl_swap_window();
    }
}
