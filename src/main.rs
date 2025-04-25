use core::f32;
use std::ffi::CStr;

use frametimer::FrameTimer;
use gamestate::{GameState, WindowData};
use imgui::ConfigFlags;
use imgui_render::Renderer;
use imgui_sdl2_support::SdlPlatform;
use sdl2::keyboard::Keycode;

const TICK_INTERVAL: f32 = 1.0 / 60.0;
const MAX_TICK_ACCUM: f32 = TICK_INTERVAL * 4.0;

extern crate sdl2;
extern crate gl;
extern crate byteorder;
extern crate basis_universal;
extern crate imgui;
extern crate imgui_sdl2_support;

pub mod math;
pub mod bsp;
pub mod misc;
pub mod graphics;
pub mod asset_loader;
pub mod serialization;
pub mod component;
pub mod system;
pub mod parse_utils;
pub mod effect;
pub mod gamestate;
pub mod imgui_render;
pub mod frametimer;

fn main() {
    let sdl = sdl2::init().unwrap();
    let sdl_video = sdl.video().unwrap();
    let sdl_timer = sdl.timer().unwrap();
    let sdl_gamecontroller = sdl.game_controller().unwrap();

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
        .window("NanoGame3D", 640, 360)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    gl::load_with(|s| sdl_video.gl_get_proc_address(s) as *const _);

    let gl_ver = unsafe { CStr::from_ptr(gl::GetString(gl::VERSION) as *const _) }.to_str().unwrap();
    let gl_renderer = unsafe { CStr::from_ptr(gl::GetString(gl::RENDERER) as *const _) }.to_str().unwrap();
    println!("{} (GL: {})", gl_renderer, gl_ver);

    unsafe {
        gl::DepthRangef(0.0, 1.0);
    }

    // init basis decoder
    basis_universal::transcoder_init();

    sdl_video.gl_set_swap_interval(1).unwrap();

    // create imgui context
    let mut imgui = imgui::Context::create();

    imgui.fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    // create imgui platform & renderer
    let mut platform = SdlPlatform::new(&mut imgui);
    let mut imgui_renderer = Renderer::new(&mut imgui);

    // crash on Pi Zero
    imgui.io_mut().config_flags.insert(ConfigFlags::NO_MOUSE_CURSOR_CHANGE);

    // create game state
    let mut game_state = GameState::new();

    let mut prev_tick = sdl_timer.performance_counter();
    let timer_freq = 1.0 / (sdl_timer.performance_frequency() as f64);
    let mut delta_accum = 0.0;

    let mut gamepad = None;

    let mut fps_timer = FrameTimer::new();
    let mut frame_timer = FrameTimer::new();

    let mut show_fps_stats = false;

    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        let frame_begin = sdl_timer.performance_counter();

        for event in event_pump.poll_iter() {
            // pass event to ImGui
            platform.handle_event(&mut imgui, &event);

            match event {
                sdl2::event::Event::Quit {..} => break 'main,
                sdl2::event::Event::ControllerDeviceAdded { timestamp: _, which } => {
                    match gamepad {
                        None => {
                            let new_gamepad = sdl_gamecontroller.open(which).unwrap();
                            println!("Opened gamepad: {}", new_gamepad.name());
                            gamepad = Some(new_gamepad);
                        }
                        _ => {
                        }
                    }
                }
                sdl2::event::Event::KeyDown { timestamp: _, window_id: _, keycode, scancode: _, keymod: _, repeat: _ } => {
                    if let Some(k) = keycode {
                        match k {
                            Keycode::F11 => {
                                show_fps_stats = !show_fps_stats;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {},
            }
        }

        // prepare new ImGui frame
        platform.prepare_frame(&mut imgui, &window, &event_pump);

        let cur_tick = sdl_timer.performance_counter();
        let diff_tick = cur_tick - prev_tick;
        let dt = ((diff_tick as f64) * timer_freq) as f32;
        prev_tick = cur_tick;

        fps_timer.update(dt, dt, |x| { 1.0 / x });

        delta_accum += dt;
        if delta_accum > MAX_TICK_ACCUM {
            delta_accum = MAX_TICK_ACCUM;
        }

        // update
        while delta_accum >= TICK_INTERVAL {
            delta_accum -= TICK_INTERVAL;
            game_state.tick(TICK_INTERVAL, gamepad.as_ref());
        }

        // render
        let win_size = window.size();
        game_state.render(WindowData { width: win_size.0 as i32, height: win_size.1 as i32 });

        let frame_end = sdl_timer.performance_counter();

        let frame_delta = frame_end - frame_begin;
        let frame_delta = ((frame_delta as f64) * timer_freq) as f32;

        frame_timer.update(frame_delta, dt, |x| { x * 1000.0 });

        // draw ImGui
        let ui = imgui.new_frame();

        if show_fps_stats {
            // overlay: framerate
            let overlay_flags = imgui::WindowFlags::NO_DECORATION |
                imgui::WindowFlags::ALWAYS_AUTO_RESIZE |
                imgui::WindowFlags::NO_SAVED_SETTINGS |
                imgui::WindowFlags::NO_FOCUS_ON_APPEARING |
                imgui::WindowFlags::NO_NAV;

            if let Some(overlay_win) = ui.window("OVERLAY")
                .position([16.0, 16.0], imgui::Condition::Always)
                .bg_alpha(0.5)
                .flags(overlay_flags)
                .begin()
            {
                ui.text(format!("Frame time (ms) - min: {}, max: {}, avg: {}",
                    (frame_timer.frame_time_min * 1000.0).round() as i32,
                    (frame_timer.frame_time_max * 1000.0).round() as i32,
                    (frame_timer.frame_time_avg * 1000.0).round() as i32));

                // graph frame time
                ui.plot_lines("##frametime_graph", &frame_timer.history)
                    .scale_max(32.0)
                    .scale_min(0.0)
                    .values_offset(0)
                    .graph_size([256.0, 64.0])
                    .build();

                ui.text(format!("FPS - min: {}, max: {}, avg: {}",
                    (1.0 / fps_timer.frame_time_min).round() as i32,
                    (1.0 / fps_timer.frame_time_max).round() as i32,
                    (1.0 / fps_timer.frame_time_avg).round() as i32));

                // graph FPS
                ui.plot_lines("##fps_graph", &fps_timer.history)
                    .scale_max(120.0)
                    .scale_min(0.0)
                    .values_offset(0)
                    .graph_size([256.0, 64.0])
                    .build();

                overlay_win.end();
            }
        }

        imgui_renderer.render(&mut imgui);

        window.gl_swap_window();
    }
}
