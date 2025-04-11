use hecs::World;

use crate::{bsp::bspcommon::{coord_space_transform, extract_frustum}, component::{camera::Camera, transform3d::Transform3D}, math::Matrix4x4, MapData, TimeData, WindowData};

/// System which performs all rendering (world + entities)
pub fn render_system(time: &TimeData, window_data: &WindowData, map_data: &mut MapData, world: &mut World) {
    // gather map models
    //let mut mapmodel_iter = world.query::<(&MapModel, &Transform3D)>();
    //let mapmodels = mapmodel_iter
    //    .iter()
    //    .collect::<Vec<_>>();

    // gather cameras
    let mut camera_iter = world.query::<(&Transform3D, &Camera)>();
    let cameras = camera_iter
        .iter()
        .collect::<Vec<_>>();

    let mut camera_index = 0;
    for (_, (transform, camera)) in cameras {
        let aspect = match camera.viewport_rect {
            Some(v) => {
                unsafe { gl::Viewport(v.x, v.y, v.w, v.h); }
                v.w as f32 / v.h as f32
            }
            None => {
                unsafe { gl::Viewport(0, 0, window_data.width, window_data.height); }
                window_data.width as f32 / window_data.height as f32
            }
        };

        // build view & projection matrices
        let cam_rot_inv = transform.rotation.inverted();

        let cam_view = Matrix4x4::translation(transform.position * -1.0)
            * Matrix4x4::rotation(cam_rot_inv);

        let cam_proj = Matrix4x4::projection_perspective(aspect, camera.fov.to_radians(), camera.near, camera.far);

        // calculate camera frustum planes
        let viewproj = cam_view * coord_space_transform() * cam_proj;

        let frustum = extract_frustum(&viewproj);

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::ClearDepth(1.0);

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // retrieve map renderer for camera
        map_data.update_renderer_cache(camera_index);
        let renderer = &mut map_data.map_renderers[camera_index];

        // update with new camera position
        renderer.update(&frustum, time.total_time, &map_data.light_layers, &map_data.map, &map_data.map_textures, &map_data.map_lightmap, transform.position);

        // draw skybox
        /*match env_data {
            Some(v) => {
                draw_env_quad(&v[0], &Quaternion::identity(), &cam_env_view, &cam_proj);
                draw_env_quad(&v[1], &Quaternion::from_euler(Vector3::new(0.0, 0.0, 180.0_f32.to_radians())), &cam_env_view, &cam_proj);
                draw_env_quad(&v[2], &Quaternion::from_euler(Vector3::new(0.0, 0.0, 90.0_f32.to_radians())), &cam_env_view, &cam_proj);
                draw_env_quad(&v[3], &Quaternion::from_euler(Vector3::new(0.0, 0.0, -90.0_f32.to_radians())), &cam_env_view, &cam_proj);
                draw_env_quad(&v[4], &Quaternion::from_euler(Vector3::new(-90.0_f32.to_radians(), 0.0, -90.0_f32.to_radians())), &cam_env_view, &cam_proj);
                draw_env_quad(&v[5], &Quaternion::from_euler(Vector3::new(90.0_f32.to_radians(), 0.0, -90.0_f32.to_radians())), &cam_env_view, &cam_proj);
            }
            _ => {
            }
        };*/

        // draw opaque geometry
        renderer.draw_opaque(&map_data.map, &map_data.map_textures, &map_data.map_lightmap, time.total_time, viewproj);

        // draw transparent geometry
        // renderer.draw_transparent(&map_data.map, &map_data.map_textures, time.total_time, &cam_view, &cam_proj);

        camera_index += 1;
    }
}