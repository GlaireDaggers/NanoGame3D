use std::{collections::HashMap, ffi::CStr, fs::File};

use asset_loader::load_model;
use bsp::{bspcommon::aabb_aabb_intersects, bspfile::BspFile, bsplightmap::BspLightmap, bsprenderer::{BspMapModelRenderer, BspMapRenderer, BspMapTextures}};
use component::{camera::{Camera, FPCamera}, charactercontroller::CharacterController, door::{Door, DoorLink, DoorOpener}, fpview::FPView, light::Light, mapmodel::MapModel, playerinput::PlayerInput, rendermesh::RenderMesh, rotator::Rotator, transform3d::Transform3D, triggerable::{TriggerLink, TriggerState}};
use hecs::{CommandBuffer, Entity, World};
use math::{Quaternion, Vector3};
use sdl2::controller::{Axis, Button, GameController};
use system::{character_system::{character_apply_input_update, character_init, character_input_update, character_rotation_update, character_update}, door_system::door_system_update, flycam_system::flycam_system_update, fpcam_system::fpcam_update, fpview_system::{fpview_eye_update, fpview_input_system_update}, render_system::{render_system, NUM_CUSTOM_LIGHT_LAYERS}, rotator_system::rotator_system_update, triggerable_system::trigger_link_system_update};

const TICK_INTERVAL: f32 = 1.0 / 60.0;
const MAX_TICK_ACCUM: f32 = TICK_INTERVAL * 4.0;

extern crate sdl2;
extern crate gl;
extern crate byteorder;
extern crate basis_universal;

pub mod math;
pub mod bsp;
pub mod misc;
pub mod graphics;
pub mod asset_loader;
pub mod component;
pub mod system;
pub mod parse_utils;

#[derive(Default)]
pub struct InputState {
    pub move_x: f32,
    pub move_y: f32,
    pub look_x: f32,
    pub look_y: f32,
    pub crouch: bool,
    pub jump: bool,
}

pub struct MapData {
    pub map: BspFile,
    pub map_textures: BspMapTextures,
    pub map_lightmap: BspLightmap,
    pub map_model_renderer: BspMapModelRenderer,
    pub map_renderers: Vec<BspMapRenderer>,
    pub light_layers: [f32;NUM_CUSTOM_LIGHT_LAYERS],
}

#[derive(Default)]
pub struct WindowData {
    pub width: i32,
    pub height: i32
}

#[derive(Default)]
pub struct TimeData {
    pub delta_time: f32,
    pub total_time: f32
}

struct GameState {
    world: World,
    time_data: TimeData,
    test_model: Entity,
    map_data: Option<MapData>,
}

impl MapData {
    pub fn load_map(map_name: &str) -> MapData {
        println!("Loading map: {}", map_name);

        let mut bsp_file = File::open(format!("content/maps/{}.bsp", map_name).as_str()).unwrap();
        let bsp = BspFile::new(&mut bsp_file);
        println!("BSP DATA LOADED");
        let bsp_textures = BspMapTextures::new(&bsp);
        println!("BSP TEXTURES LOADED");
        let bsp_lightmap = BspLightmap::new(&bsp);
        println!("LIGHTMAP ATLAS CREATED");
        let bsp_map_model_renderer = BspMapModelRenderer::new(&bsp, &bsp_textures, &bsp_lightmap);
        println!("MAP MODEL RENDERER CREATED");

        println!("Map loaded");

        MapData {
            map: bsp,
            map_textures: bsp_textures,
            map_renderers: Vec::new(),
            map_lightmap: bsp_lightmap,
            map_model_renderer: bsp_map_model_renderer,
            light_layers: [0.0;NUM_CUSTOM_LIGHT_LAYERS]
        }
    }

    pub fn update_renderer_cache(self: &mut Self, index: usize) {
        while self.map_renderers.len() <= index {
            println!("Allocating map renderer for camera {}", index);
            self.map_renderers.push(BspMapRenderer::new(&self.map));
        }
    }
}

impl GameState {
    pub fn new() -> GameState {
        let mut world = World::new();

        let map_data = MapData::load_map("e1m1");

        let mut player_start_pos = Vector3::zero();
        let mut player_start_rot = 0.0;

        let mut targetmap = HashMap::new();
        let mut pending_resolve_targets = Vec::new();

        let mut doors = Vec::new();

        // spawn entities
        map_data.map.entity_lump.parse(|entity_data| {
            match entity_data["classname"] {
                "info_player_start" => {
                    player_start_pos = parse_utils::parse_prop_vec3(&entity_data, "origin", Vector3::zero());
                    player_start_rot = parse_utils::parse_prop::<f32>(&entity_data, "angle", 0.0) + 180.0;
                }
                "worldspawn" => {
                    for (key, val) in entity_data {
                        println!("worldspawn: {} = {}", key, val);
                    }
                }
                "prop_static" => {
                    let pos = parse_utils::parse_prop_vec3(&entity_data, "origin", Vector3::zero());
                    let angles = parse_utils::parse_prop_vec3(&entity_data, "angles", Vector3::zero());
                    let model_path = parse_utils::get_prop_str(&entity_data, "model", "");
                    let scale = parse_utils::parse_prop_vec3(&entity_data, "scale", Vector3::new(1.0, 1.0, 1.0));

                    let rot = Quaternion::from_euler(Vector3::new(angles.x.to_radians(), angles.z.to_radians(), angles.y.to_radians()));
                    let model = load_model(format!("content/{}", model_path).as_str()).unwrap();

                    world.spawn((
                        Transform3D::default().with_position(pos).with_rotation(rot).with_scale(scale),
                        RenderMesh::new(model),
                    ));
                }
                "light" => {
                    let light_pos = parse_utils::parse_prop_vec3(&entity_data, "origin", Vector3::zero());
                    let light_intensity = parse_utils::parse_prop::<f32>(&entity_data, "light", 300.0);
                    let light_color = parse_utils::parse_prop_vec3(&entity_data, "_color", Vector3::new(1.0, 1.0, 1.0));

                    world.spawn((
                        Transform3D::default().with_position(light_pos),
                        Light { color: light_color, max_radius: light_intensity }
                    ));
                }
                "func_door" => {
                    let model_idx = parse_utils::parse_prop_modelindex(&entity_data, "model", usize::MAX);
                    let submodel = &map_data.map.submodel_lump.submodels[model_idx + 1];
                    let pos = submodel.origin;
                    let size = submodel.maxs - submodel.mins;

                    let target_name = parse_utils::get_prop_str(&entity_data, "targetname", "");
                    let target = parse_utils::get_prop_str(&entity_data, "target", "");

                    let auto_open = target_name == "";

                    let angle = parse_utils::parse_prop::<i32>(&entity_data, "angle", 0);
                    let speed = parse_utils::parse_prop::<f32>(&entity_data, "speed", 100.0);
                    let lip = parse_utils::parse_prop::<f32>(&entity_data, "lip", 0.0);

                    let spawn_flags = parse_utils::parse_prop::<u32>(&entity_data, "spawnflags", 0);

                    let move_dir = if angle == -1 {
                        Vector3::new(0.0, 0.0, 1.0)
                    }
                    else if angle == -2 {
                        Vector3::new(0.0, 0.0, -1.0)
                    }
                    else {
                        let r = (angle as f32).to_radians();
                        let sx = r.cos();
                        let sy = r.sin();

                        Vector3::new(sx, sy, 0.0)
                    };

                    // calculate move distance along direction
                    let move_dist = (move_dir.x.abs() * size.x +
                        move_dir.y.abs() * size.y +
                        move_dir.z.abs() * size.z) - lip;

                    let open_pos = pos + (move_dir * move_dist);

                    let e = world.spawn((
                        Transform3D::default().with_position(pos),
                        Door { auto_open, open_pos, close_pos: pos, move_speed: speed },
                        TriggerState { triggered: false },
                        MapModel { model_idx }
                    ));

                    if target != "" {
                        pending_resolve_targets.push((e, target.to_owned()));
                    }

                    if target_name != "" {
                        targetmap.insert(target_name.to_owned(), e);
                    }

                    // don't link doors if they have the "don't link" spawn flag set
                    if spawn_flags & 4 == 0 {
                        doors.push((e, submodel));
                    }
                }
                "func_explosive" => {
                    let model_idx = parse_utils::parse_prop_modelindex(&entity_data, "model", usize::MAX);
                    let submodel = &map_data.map.submodel_lump.submodels[model_idx + 1];
                    let pos = submodel.origin;
                    
                    world.spawn((
                        Transform3D::default().with_position(pos),
                        MapModel { model_idx }
                    ));
                }
                "func_wall" => {
                    let model_idx = parse_utils::parse_prop_modelindex(&entity_data, "model", usize::MAX);
                    let submodel = &map_data.map.submodel_lump.submodels[model_idx + 1];
                    let pos = submodel.origin;
                    
                    world.spawn((
                        Transform3D::default().with_position(pos),
                        MapModel { model_idx }
                    ));
                }
                "func_object" => {
                    let model_idx = parse_utils::parse_prop_modelindex(&entity_data, "model", usize::MAX);
                    let submodel = &map_data.map.submodel_lump.submodels[model_idx + 1];
                    let pos = submodel.origin;
                    
                    world.spawn((
                        Transform3D::default().with_position(pos),
                        MapModel { model_idx }
                    ));
                }
                "func_plat" => {
                    let model_idx = parse_utils::parse_prop_modelindex(&entity_data, "model", usize::MAX);
                    let submodel = &map_data.map.submodel_lump.submodels[model_idx + 1];
                    let pos = submodel.origin;
                    
                    world.spawn((
                        Transform3D::default().with_position(pos),
                        MapModel { model_idx }
                    ));
                }
                "func_rotating" => {
                    let model_idx = parse_utils::parse_prop_modelindex(&entity_data, "model", usize::MAX);
                    let submodel = &map_data.map.submodel_lump.submodels[model_idx + 1];
                    let spawn_flags = parse_utils::parse_prop::<u32>(&entity_data, "spawnflags", 0);
                    let pos = parse_utils::parse_prop_vec3(&entity_data, "origin", submodel.origin);
                    let speed = parse_utils::parse_prop::<f32>(&entity_data, "speed", 0.0);

                    let axis = if spawn_flags & 4 != 0 {
                        Vector3::unit_x()
                    }
                    else if spawn_flags & 8 != 0 {
                        Vector3::unit_y()
                    }
                    else {
                        Vector3::unit_z()
                    };
                    
                    world.spawn((
                        Transform3D::default().with_position(pos),
                        Rotator { rot_axis: axis, rot_speed: speed },
                        MapModel { model_idx }
                    ));
                }
                "func_train" => {
                    let model_idx = parse_utils::parse_prop_modelindex(&entity_data, "model", usize::MAX);
                    let submodel = &map_data.map.submodel_lump.submodels[model_idx + 1];
                    let pos = submodel.origin;
                    
                    world.spawn((
                        Transform3D::default().with_position(pos),
                        MapModel { model_idx }
                    ));
                }
                _ => {
                }
            }
        });

        // resolve triggerable entity targets
        let mut cmd_buf = CommandBuffer::new();
        for (e, targetname) in pending_resolve_targets {
            if !targetmap.contains_key(&targetname) {
                println!("Couldn't find trigger target: {}", &targetname);
            }
            else {
                let target_ent = targetmap[&targetname];
                cmd_buf.insert_one(e, TriggerLink {
                    target: target_ent
                });
            }
        }
        cmd_buf.run_on(&mut world);

        // link doors together if they are touching
        let mut pending_door_links = Vec::new();
        for (e, doormodel) in &doors {
            let mut links = Vec::new();
            for (e2, doormodel2) in &doors {
                if e2 != e && aabb_aabb_intersects(doormodel.mins, doormodel.maxs, doormodel2.mins, doormodel2.maxs) {
                    links.push(*e2);
                }
            }
            pending_door_links.push((e, links));
        }

        for (e, links) in pending_door_links {
            cmd_buf.insert_one(*e, DoorLink {
                links
            });
        }

        cmd_buf.run_on(&mut world);

        // player & camera
        let player_entity = world.spawn((
            Transform3D::default().with_position(player_start_pos),
            FPView::new(-player_start_rot, 0.0, 40.0),
            CharacterController::default(),
            PlayerInput::new(),
            DoorOpener {},
            Light { max_radius: 200.0, color: Vector3::new(1.0, 1.0, 1.0) }
        ));

        world.spawn((
            Transform3D::default(),
            Camera::default(),
            FPCamera::new(player_entity)
        ));

        // test static model entity
        let test_model = world.spawn((
            Transform3D::default().with_position(Vector3::new(0.0, 0.0, 50.0)).with_scale(Vector3::new(100.0, 100.0, 100.0)),
            RenderMesh::new(load_model("content/models/dragon-2_80.glb").unwrap()),
            Rotator { rot_axis: Vector3::unit_z(), rot_speed: 45.0_f32.to_radians() }
        ));

        GameState {
            world,
            time_data: TimeData::default(),
            map_data: Some(map_data),
            test_model,
        }
    }

    pub fn tick(self: &mut Self, delta: f32, gamepad: Option<&GameController>) {
        let mut input_state = InputState {
            move_x: 0.0,
            move_y: 0.0,
            look_x: 0.0,
            look_y: 0.0,
            crouch: false,
            jump: false
        };

        if let Some(gp) = gamepad {
            input_state.move_x = gp.axis(Axis::LeftX) as f32 / 32767.0;
            input_state.move_y = gp.axis(Axis::LeftY) as f32 / -32767.0;
            input_state.look_x = gp.axis(Axis::RightX) as f32 / 32767.0;
            input_state.look_y = gp.axis(Axis::RightY) as f32 / -32767.0;
            input_state.jump = gp.button(Button::A);
            input_state.crouch = gp.button(Button::B);
        }

        // update time
        self.time_data.delta_time = delta;
        self.time_data.total_time += delta;

        {
            let mut test_model_transform = self.world.get::<&mut Transform3D>(self.test_model).unwrap();
            test_model_transform.position = Vector3::new((self.time_data.total_time * 0.1).sin() * 150.0, (self.time_data.total_time * 0.25).sin() * 150.0, 50.0);
        }

        // update
        if let Some(map_data) = &mut self.map_data {
            rotator_system_update(&self.time_data, &mut self.world);
            door_system_update(&self.time_data, map_data, &mut self.world);
            trigger_link_system_update(&mut self.world);
            fpview_input_system_update(&input_state, &self.time_data, &mut self.world);
            character_init(&mut self.world);
            character_rotation_update(&mut self.world);
            character_input_update(&input_state, &mut self.world);
            fpview_eye_update(&self.time_data, &mut self.world);
            character_apply_input_update(&self.time_data, map_data, &mut self.world);
            character_update(&self.time_data, map_data, &mut self.world);
            flycam_system_update(&input_state, &self.time_data, &map_data.map, &mut self.world);
            fpcam_update(&mut self.world);
        }
    }

    pub fn render(self: &mut Self, window_data: WindowData) {
        // render
        if let Some(map_data) = &mut self.map_data {
            render_system(&self.time_data, &window_data, map_data, &mut self.world);
        }
    }
}

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

    // create game state
    let mut game_state = GameState::new();

    let mut prev_tick = sdl_timer.performance_counter();
    let timer_freq = 1.0 / (sdl_timer.performance_frequency() as f64);
    let mut delta_accum = 0.0;

    let mut gamepad = None;

    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
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
                _ => {},
            }
        }

        let cur_tick = sdl_timer.performance_counter();
        let diff_tick = cur_tick - prev_tick;
        let dt = ((diff_tick as f64) * timer_freq) as f32;
        prev_tick = cur_tick;

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

        window.gl_swap_window();
    }
}
