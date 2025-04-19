use hecs::World;
use lazy_static::lazy_static;

use crate::{bsp::{bspcommon::{aabb_frustum, coord_space_transform, extract_frustum}, bspfile::LSHProbeSample}, component::{camera::Camera, mapmodel::MapModel, meshpose::MeshPose, rendermesh::{RenderMesh, SkinnedMesh}, transform3d::Transform3D}, graphics::model::{MeshVertex, ModelSkin}, math::{Matrix4x4, Vector4}, MapData, TimeData, WindowData};

pub const NUM_CUSTOM_LIGHT_LAYERS: usize = 30;
pub const CUSTOM_LIGHT_LAYER_START: usize = 32;
pub const CUSTOM_LIGHT_LAYER_END: usize = CUSTOM_LIGHT_LAYER_START + NUM_CUSTOM_LIGHT_LAYERS;

lazy_static! {
    static ref LIGHTSTYLES: [Vec<f32>;13] = [
        make_light_table(b"m"),
        // 1 - FLICKER 1
        make_light_table(b"mmnmmommommnonmmonqnmmo"),
        // 2 - SLOW STRONG PULSE
        make_light_table(b"abcdefghijklmnopqrstuvwxyzyxwvutsrqponmlkjihgfedcba"),
        // 3 - CANDLE 1
        make_light_table(b"mmmmmaaaaammmmmaaaaaabcdefgabcdefg"),
        // 4 - FAST STROBE
        make_light_table(b"mamamamamama"),
        // 5 - GENTLE PULSE
        make_light_table(b"jklmnopqrstuvwxyzyxwvutsrqponmlkj"),
        // 6 - FLICKER 2
        make_light_table(b"nmonqnmomnmomomno"),
        // 7 - CANDLE 2
        make_light_table(b"mmmaaaabcdefgmmmmaaaammmaamm"),
        // 8 - CANDLE 3
        make_light_table(b"mmmaaammmaaammmabcdefaaaammmmabcdefmmmaaaa"),
        // 9 - SLOW STROBE
        make_light_table(b"aaaaaaaazzzzzzzz"),
        // 10 - FLUORESCENT FLICKER
        make_light_table(b"mmamammmmammamamaaamammma"),
        // 11 - SLOW PULSE, NO BLACK
        make_light_table(b"abcdefghijklmnopqrrqponmlkjihgfedcba"),
        // 12 - FAST PULSE
        make_light_table(b"acegikmoqsuwyywusqomkigeca"),
    ];
}

// convert Quake-style light animation table to float array ('a' is minimum light, 'z' is maximum light)
fn make_light_table(data: &[u8]) -> Vec<f32> {
    let mut output = vec![0.0;data.len()];

    for i in 0..data.len() {
        output[i] = (data[i] - 97) as f32 / 25.0;
    }

    output
}

fn draw_mesh_iter(mesh: &RenderMesh, transparent: bool, cur_node: &mut usize, parent_transform: Matrix4x4, viewproj: &Matrix4x4, sh: &LSHProbeSample) {
    let node = &mesh.mesh.nodes[*cur_node];
    let node_xform = node.transform * parent_transform;
    let mvp = node_xform * *viewproj;

    if node.mesh_index >= 0 {
        // draw mesh attached to node
        let node_mesh = &mesh.mesh.meshes[node.mesh_index as usize];

        for part in &node_mesh.parts {
            if let Some((vtx_buffer, idx_buffer)) = &part.buffers {
                let mat = &mesh.mesh.materials[part.material_index];

                if mat.transparent == transparent {
                    mat.apply();

                    mat.shader.set_uniform_vec4("shR", sh.sh_r);
                    mat.shader.set_uniform_vec4("shG", sh.sh_g);
                    mat.shader.set_uniform_vec4("shB", sh.sh_b);
                    mat.shader.set_uniform_mat4("localToWorld", node_xform);
                    mat.shader.set_uniform_mat4("mvp", mvp);

                    unsafe {
                        gl::FrontFace(part.winding);

                        gl::BindBuffer(gl::ARRAY_BUFFER, vtx_buffer.handle());
                        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, idx_buffer.handle());

                        MeshVertex::setup_vtx_arrays(&mat.shader);

                        // draw geometry
                        gl::DrawElements(part.topology, part.indices.len() as i32, gl::UNSIGNED_SHORT, 0 as *const _);
                    }
                }
            }
        }
    }

    *cur_node += 1;

    // draw children
    for _ in 0..node.num_children {
        draw_mesh_iter(mesh, transparent, cur_node, node_xform, viewproj, sh);
    }
}

fn do_skinning(vertices: &mut [MeshVertex], node_transforms: &[Matrix4x4], skin: &ModelSkin) {
    for vtx in vertices {
        #[cfg(feature = "two_bone_per_vertex")]
        let tx = {
            let n0 = &skin.joints[vtx.joints[0] as usize];
            let n1 = &skin.joints[vtx.joints[1] as usize];
    
            let w0 = vtx.weights[0] as f32 / 255.0;
            let w1 = vtx.weights[1] as f32 / 255.0;
    
            let t0 = n0.inv_bind_xform * node_transforms[n0.node];
            let t1 = n1.inv_bind_xform * node_transforms[n1.node];
    
            (t0 * w0) + (t1 * w1)
        };

        #[cfg(not(feature = "two_bone_per_vertex"))]
        let tx = {
            let n0 = &skin.joints[vtx.joints[0] as usize];
            let n1 = &skin.joints[vtx.joints[1] as usize];
            let n2 = &skin.joints[vtx.joints[2] as usize];
            let n3 = &skin.joints[vtx.joints[3] as usize];
    
            let w0 = vtx.weights[0] as f32 / 255.0;
            let w1 = vtx.weights[1] as f32 / 255.0;
            let w2 = vtx.weights[2] as f32 / 255.0;
            let w3 = vtx.weights[3] as f32 / 255.0;
    
            let t0 = n0.inv_bind_xform * node_transforms[n0.node];
            let t1 = n1.inv_bind_xform * node_transforms[n1.node];
            let t2 = n2.inv_bind_xform * node_transforms[n2.node];
            let t3 = n3.inv_bind_xform * node_transforms[n3.node];
    
            (t0 * w0) + (t1 * w1) + (t2 * w2) + (t3 * w3)
        };

        let pos = tx * Vector4::new(vtx.position.x, vtx.position.y, vtx.position.z, 1.0);
        let nrm = tx * Vector4::new(vtx.normal.x, vtx.normal.y, vtx.normal.z, 0.0);
        let tan = tx * Vector4::new(vtx.tangent.x, vtx.tangent.y, vtx.tangent.z, 0.0);

        vtx.position.x = pos.x;
        vtx.position.y = pos.y;
        vtx.position.z = pos.z;

        vtx.normal.x = nrm.x;
        vtx.normal.y = nrm.y;
        vtx.normal.z = nrm.z;

        vtx.tangent.x = tan.x;
        vtx.tangent.y = tan.y;
        vtx.tangent.z = tan.z;
    }
}

fn update_skinned_mesh_iter(mesh: &RenderMesh, pose: &MeshPose, sk: &mut SkinnedMesh, cur_node: &mut usize) {
    let node = &mesh.mesh.nodes[*cur_node];

    if node.mesh_index >= 0 {
        // draw mesh attached to node
        let node_mesh = &mesh.mesh.meshes[node.mesh_index as usize];

        for (part_idx, part) in node_mesh.parts.iter().enumerate() {
            if node.skin_index >= 0 {
                // apply skinning
                sk.vtx_array.resize(part.vertices.len(), MeshVertex::default());
                sk.vtx_array.copy_from_slice(&part.vertices);
                do_skinning(&mut sk.vtx_array, &pose.pose, &mesh.mesh.skins[node.skin_index as usize]);

                let buf = &mut sk.vtx_buffer[node.mesh_index as usize][part_idx];

                // intentionally orphan buffer so we can keep reusing it
                buf.resize((sk.vtx_array.len() * size_of::<MeshVertex>()) as isize);
                buf.set_data(0, &sk.vtx_array);
            }
        }
    }

    *cur_node += 1;

    // draw children
    for _ in 0..node.num_children {
        update_skinned_mesh_iter(mesh, pose, sk, cur_node);
    }
}

fn draw_skinned_mesh_iter(mesh: &RenderMesh, pose: &MeshPose, sk: &SkinnedMesh, transparent: bool, cur_node: &mut usize, parent_transform: Matrix4x4, viewproj: &Matrix4x4, sh: &LSHProbeSample) {
    let node = &mesh.mesh.nodes[*cur_node];
    let node_xform = pose.pose[*cur_node] * parent_transform;
    let mvp = node_xform * *viewproj;

    if node.mesh_index >= 0 {
        // draw mesh attached to node
        let node_mesh = &mesh.mesh.meshes[node.mesh_index as usize];

        for (part_idx, part) in node_mesh.parts.iter().enumerate() {
            if let Some((vtx_buffer, idx_buffer)) = &part.buffers {
                let mat = &mesh.mesh.materials[part.material_index];

                if mat.transparent == transparent {
                    let vtx_buffer = if node.skin_index >= 0 {
                        &sk.vtx_buffer[node.mesh_index as usize][part_idx]
                    }
                    else {
                        vtx_buffer
                    };

                    mat.apply();

                    mat.shader.set_uniform_vec4("shR", sh.sh_r);
                    mat.shader.set_uniform_vec4("shG", sh.sh_g);
                    mat.shader.set_uniform_vec4("shB", sh.sh_b);
                    mat.shader.set_uniform_mat4("localToWorld", node_xform);
                    mat.shader.set_uniform_mat4("mvp", mvp);

                    unsafe {
                        gl::FrontFace(part.winding);

                        gl::BindBuffer(gl::ARRAY_BUFFER, vtx_buffer.handle());
                        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, idx_buffer.handle());

                        MeshVertex::setup_vtx_arrays(&mat.shader);

                        // draw geometry
                        gl::DrawElements(part.topology, part.indices.len() as i32, gl::UNSIGNED_SHORT, 0 as *const _);
                    }
                }
            }
        }
    }

    *cur_node += 1;

    // draw children
    for _ in 0..node.num_children {
        draw_skinned_mesh_iter(mesh, pose, sk, transparent, cur_node, parent_transform, viewproj, sh);
    }
}

/// System which performs vertex skinning
pub fn skinning_system(world: &mut World) {
    let mut sk_mesh_iter = world.query::<(&RenderMesh, &mut SkinnedMesh, &MeshPose)>();

    for (_, (mesh, sk, pose)) in sk_mesh_iter.iter() {
        let mut cur_node = 0;
        while cur_node < mesh.mesh.nodes.len() {
            update_skinned_mesh_iter(mesh, pose, sk, &mut cur_node);
        }
    }
}

/// System which performs all rendering (world + entities)
pub fn render_system(time: &TimeData, window_data: &WindowData, map_data: &mut MapData, world: &mut World) {
    // gather map models
    let mut mapmodel_iter = world.query::<(&MapModel, &Transform3D)>();
    let mapmodels = mapmodel_iter
        .iter()
        .collect::<Vec<_>>();

    // gather static meshes
    let mut mesh_iter = world.query::<(&RenderMesh, &Transform3D)>().without::<&SkinnedMesh>();
    let meshes = mesh_iter
        .iter()
        .collect::<Vec<_>>();

    // gather skinned meshes
    let mut sk_mesh_iter = world.query::<(&RenderMesh, &Transform3D, &SkinnedMesh, &MeshPose)>();
    let sk_meshes = sk_mesh_iter
        .iter()
        .collect::<Vec<_>>();

    // gather cameras
    let mut camera_iter = world.query::<(&Transform3D, &Camera)>();
    let cameras = camera_iter
        .iter()
        .collect::<Vec<_>>();

    // compute light layers
    let lightstyle_frame = (time.total_time * 10.0) as usize;
    let lightstyle_frame_lerp = (time.total_time * 10.0).fract();
    let mut light_styles = [0.0;256];

    for (idx, tbl) in LIGHTSTYLES.iter().enumerate() {
        let a = tbl[lightstyle_frame % tbl.len()];
        let b = tbl[(lightstyle_frame + 1) % tbl.len()];
        light_styles[idx] = (a * (1.0 - lightstyle_frame_lerp)) + (b * lightstyle_frame_lerp);
    }

    for (idx, sc) in map_data.light_layers.iter().enumerate() {
        light_styles[idx + CUSTOM_LIGHT_LAYER_START] = *sc;
    }

    // draw cameras
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
        map_data.map_model_renderer.update(&light_styles, &visible_model_indices);

        // update with new camera position
        renderer.update(&frustum, &light_styles, &map_data.map, &map_data.map_textures, &map_data.map_lightmap, transform.position);

        // draw opaque map geometry
        renderer.draw_opaque(&map_data.map_textures, &map_data.map_lightmap, time.total_time, viewproj);

        for (idx, transform) in visible_model_indices.iter().zip(&visible_model_transforms) {
            map_data.map_model_renderer.draw_model(false, &map_data.map_textures, &map_data.map_lightmap, *idx, *transform, viewproj);
        }

        // draw opaque mesh parts
        for (_, (mesh, transform)) in &meshes {
            let sh = map_data.map.lsh_grid_lump.sample_position(transform.position, &light_styles);

            let model_transform = mesh.mesh.root_transform
                * Matrix4x4::scale(transform.scale)
                * Matrix4x4::rotation(transform.rotation)
                * Matrix4x4::translation(transform.position);
            
            let mut cur_node = 0;
            while cur_node < mesh.mesh.nodes.len() {
                draw_mesh_iter(mesh, false, &mut cur_node, model_transform, &viewproj, &sh);
            }
        }

        for (_, (mesh, transform, sk, pose)) in &sk_meshes {
            let sh = map_data.map.lsh_grid_lump.sample_position(transform.position, &light_styles);

            let model_transform = mesh.mesh.root_transform
                * Matrix4x4::scale(transform.scale)
                * Matrix4x4::rotation(transform.rotation)
                * Matrix4x4::translation(transform.position);
            
            let mut cur_node = 0;
            while cur_node < mesh.mesh.nodes.len() {
                draw_skinned_mesh_iter(mesh, pose, sk, false, &mut cur_node, model_transform, &viewproj, &sh);
            }
        }

        // draw transparent map geometry
        renderer.draw_transparent(&map_data.map_textures, &map_data.map_lightmap, time.total_time, viewproj);

        for (idx, transform) in visible_model_indices.iter().zip(&visible_model_transforms) {
            map_data.map_model_renderer.draw_model(true, &map_data.map_textures, &map_data.map_lightmap, *idx, *transform, viewproj);
        }

        // draw transparent mesh parts
        // TODO: this feels super inefficient for deep hierarchies. Maybe just avoid those?
        for (_, (mesh, transform)) in &meshes {
            let sh = map_data.map.lsh_grid_lump.sample_position(transform.position, &light_styles);

            let model_transform = mesh.mesh.root_transform
                * Matrix4x4::scale(transform.scale)
                * Matrix4x4::rotation(transform.rotation)
                * Matrix4x4::translation(transform.position);
            
            let mut cur_node = 0;
            while cur_node < mesh.mesh.nodes.len() {
                draw_mesh_iter(mesh, true, &mut cur_node, model_transform, &viewproj, &sh);
            }
        }

        for (_, (mesh, transform, sk, pose)) in &sk_meshes {
            let sh = map_data.map.lsh_grid_lump.sample_position(transform.position, &light_styles);

            let model_transform = mesh.mesh.root_transform
                * Matrix4x4::scale(transform.scale)
                * Matrix4x4::rotation(transform.rotation)
                * Matrix4x4::translation(transform.position);
            
            let mut cur_node = 0;
            while cur_node < mesh.mesh.nodes.len() {
                draw_skinned_mesh_iter(mesh, pose, sk, true, &mut cur_node, model_transform, &viewproj, &sh);
            }
        }

        camera_index += 1;
    }
}