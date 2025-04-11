use hecs::World;

use crate::{bsp::bspcommon::{aabb_frustum, coord_space_transform, extract_frustum}, component::{camera::Camera, mapmodel::MapModel, transform3d::Transform3D}, math::Matrix4x4, MapData, TimeData, WindowData};

/// System which performs all rendering (world + entities)
pub fn render_system(time: &TimeData, window_data: &WindowData, map_data: &mut MapData, world: &mut World) {
    // gather map models
    let mut mapmodel_iter = world.query::<(&MapModel, &Transform3D)>();
    let mapmodels = mapmodel_iter
        .iter()
        .collect::<Vec<_>>();

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

            // hate this stupid API man
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            gl::DepthMask(gl::TRUE);

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // retrieve map renderer for camera
        map_data.update_renderer_cache(camera_index);
        let renderer = &mut map_data.map_renderers[camera_index];

        // gather visible models
        let mut visible_model_transforms = Vec::new();
        let mut visible_model_indices = Vec::new();
        for (_, (model_info, model_transform)) in &mapmodels {
            let submodel = &map_data.map.submodel_lump.submodels[model_info.model_idx + 1];
            let bounds_extents = (submodel.maxs - submodel.mins) * 0.5;
            let bounds_center = model_transform.position + ((submodel.maxs + submodel.mins) * 0.5);

            let vis = aabb_frustum(bounds_center - bounds_extents, bounds_center + bounds_extents, &frustum) && renderer.check_vis(&map_data.map, bounds_center, bounds_extents);

            if vis {
                let model_mat = Matrix4x4::scale(model_transform.scale)
                    * Matrix4x4::rotation(model_transform.rotation)
                    * Matrix4x4::translation(model_transform.position);

                visible_model_transforms.push(model_mat);
                visible_model_indices.push(model_info.model_idx);
            }
        }

        // update models
        map_data.map_model_renderer.update(&map_data.light_layers, &visible_model_indices, time.total_time);

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

        // draw opaque map geometry
        renderer.draw_opaque(&map_data.map, &map_data.map_textures, &map_data.map_lightmap, time.total_time, viewproj);

        for (idx, transform) in visible_model_indices.iter().zip(&visible_model_transforms) {
            map_data.map_model_renderer.draw_model_opaque(&map_data.map, &map_data.map_textures, &map_data.map_lightmap, *idx, *transform, viewproj);
        }

        // draw transparent map geometry
        renderer.draw_transparent(&map_data.map, &map_data.map_textures, &map_data.map_lightmap, time.total_time, viewproj);

        for (idx, transform) in visible_model_indices.iter().zip(&visible_model_transforms) {
            map_data.map_model_renderer.draw_model_transparent(&map_data.map, &map_data.map_textures, &map_data.map_lightmap, *idx, *transform, viewproj);
        }

        camera_index += 1;
    }
}