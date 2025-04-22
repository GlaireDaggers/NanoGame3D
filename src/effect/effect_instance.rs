use std::{mem::offset_of, ops::Range, sync::Arc};

use rand::{distr::uniform::SampleUniform, rngs::ThreadRng, Rng};

use crate::{bsp::bspcommon::coord_space_transform, graphics::{buffer::Buffer, shader::Shader}, math::{Matrix4x4, Quaternion, Vector2, Vector3, Vector4}, misc::Color32};

use super::effect_data::{EffectData, EffectDisplay, EffectEmitter};

#[derive(Clone, Copy)]
pub struct ParticleVertex {
    pub position: Vector4,
    pub texcoord: Vector2,
    pub color: Color32
}

#[derive(Clone, Copy)]
struct Particle {
    pub lifetime: f32,
    pub lifetime_delta: f32,
    pub position: Vector3,
    pub angle: f32,
    pub angle_axis: Vector3,
    pub velocity: Vector3,
    pub angular_velocity: f32,
    pub scale: f32,
}

enum EffectEmitterRenderer {
    None,
    Sprite {
        vertices: Vec<ParticleVertex>,
        vertex_buffer: Buffer,
        index_buffer: Buffer,
    }
}

struct EffectEmitterInstance {
    pub transform: Matrix4x4,
    pub particles: Vec<Particle>,
    pub renderer: EffectEmitterRenderer,
    emit_timer: f32,
    burst_count: u32,
}

pub struct EffectInstance {
    pub transform: Matrix4x4,
    pub effect_data: Arc<EffectData>,
    pub enable_emit: bool,
    emitters: Vec<EffectEmitterInstance>
}

impl ParticleVertex {
    pub fn setup_vtx_arrays(shader: &Shader) {
        let position = shader.get_attribute_location("in_position");
        let texcoord = shader.get_attribute_location("in_texcoord");
        let color = shader.get_attribute_location("in_color");

        unsafe {
            gl::EnableVertexAttribArray(position);
            gl::EnableVertexAttribArray(texcoord);
            gl::EnableVertexAttribArray(color);

            gl::VertexAttribPointer(position, 4, gl::FLOAT, gl::FALSE, size_of::<ParticleVertex>() as i32, offset_of!(ParticleVertex, position) as *const _);
            gl::VertexAttribPointer(texcoord, 2, gl::FLOAT, gl::FALSE, size_of::<ParticleVertex>() as i32, offset_of!(ParticleVertex, texcoord) as *const _);
            gl::VertexAttribPointer(color, 4, gl::UNSIGNED_BYTE, gl::TRUE, size_of::<ParticleVertex>() as i32, offset_of!(ParticleVertex, color) as *const _);
        }
    }
}

impl EffectEmitterInstance {
    fn random_range<T>(rng: &mut ThreadRng, range: Range<T>) -> T where T : SampleUniform + PartialOrd {
        if range.is_empty() {
            return range.start;
        }

        rng.random_range::<T, Range<T>>(range)
    }

    fn random_axis(rng: &mut ThreadRng, dir: Vector3, spread: f32) -> Vector3 {
        let r = Quaternion::from_euler(Vector3::new(
            Self::random_range(rng, -spread .. spread),
            Self::random_range(rng, -spread .. spread),
            Self::random_range(rng, -spread .. spread)
        ));

        return r * dir;
    }

    fn emit_particle(self: &mut EffectEmitterInstance, data: &EffectEmitter, parent_transform: Matrix4x4, rng: &mut ThreadRng) -> Particle {
        let xform = self.transform * parent_transform;

        let position = match data.emit.shape {
            super::effect_data::EffectEmissionShape::Point { origin } => self.transform.transform_point(origin),
            super::effect_data::EffectEmissionShape::Box { origin, extents } => {
                let pos = Vector3::new(
                    origin.x + Self::random_range(rng, -extents.x .. extents.x),
                    origin.y + Self::random_range(rng, -extents.y .. extents.y),
                    origin.z + Self::random_range(rng, -extents.z .. extents.z) 
                );

                xform.transform_point(pos)
            },
            super::effect_data::EffectEmissionShape::Sphere { origin, inner_radius, outer_radius } => {
                let radius = Self::random_range(rng, inner_radius .. outer_radius);
                let dir = Vector3::new(
                    Self::random_range(rng, -1.0 .. 1.0),
                    Self::random_range(rng, -1.0 .. 1.0),
                    Self::random_range(rng, -1.0 .. 1.0)
                ).normalized();

                let pos = origin + (dir * radius);
                xform.transform_point(pos)
            },
            super::effect_data::EffectEmissionShape::Ring { origin, axis, inner_radius, outer_radius } => {
                // pick a pair of perpendicular basis vectors
                let (bx, by) = if axis.x.abs() <= f32::EPSILON && axis.y.abs() <= f32::EPSILON && axis.z.abs() > f32::EPSILON {
                    (Vector3::unit_x(), Vector3::unit_y())
                }
                else {
                    let a = axis.cross(Vector3::unit_z()).normalized();
                    let b = axis.cross(a);
                    (a, b)
                };

                // calculate offset within circle
                let radius = Self::random_range(rng, inner_radius .. outer_radius);
                let dir = Vector2::new(
                    Self::random_range(rng, -1.0 .. 1.0),
                    Self::random_range(rng, -1.0 .. 1.0)
                ).normalized();

                // multiply by basis vectors
                let dx = bx * dir.x;
                let dy = by * dir.y;

                let pos = origin + (dx * radius) + (dy * radius);
                xform.transform_point(pos)
            },
        };

        let lifetime = Self::random_range(rng, data.init.lifetime_min .. data.init.lifetime_max);
        let angle = Self::random_range(rng, data.init.angle_min .. data.init.angle_max);
        let angle_axis = Self::random_axis(rng, data.init.angle_axis, data.init.angle_axis_spread.to_radians()).normalized();
        let direction = Self::random_axis(rng, data.init.direction, data.init.direction_spread.to_radians()).normalized();
        let velocity = direction * Self::random_range(rng, data.init.velocity_min .. data.init.velocity_max);
        let angular_velocity = Self::random_range(rng, data.init.angular_velocity_min .. data.init.angular_velocity_max);
        let scale = Self::random_range(rng, data.init.scale_min .. data.init.scale_max);

        Particle {
            lifetime: 0.0,
            lifetime_delta: 1.0 / lifetime,
            position,
            angle,
            angle_axis,
            velocity,
            angular_velocity,
            scale
        }
    }

    pub fn update_emit(self: &mut EffectEmitterInstance, data: &EffectEmitter, parent_transform: Matrix4x4, rng: &mut ThreadRng, delta: f32) {
        // don't emit if we've hit max burst count
        if let Some(max_burst) = data.emit.max_bursts {
            if self.burst_count >= max_burst {
                return;
            }
        }

        if data.emit.max_particles == 0 {
            return;
        }

        self.emit_timer += delta;

        while self.emit_timer >= data.emit.burst_interval {
            // emit new particles
            for _ in 0..data.emit.particles_per_burst {
                let p = self.emit_particle(data, parent_transform, rng);

                if self.particles.len() >= data.emit.max_particles as usize {
                    // scan for oldest
                    let oldest = self.particles.iter().enumerate().min_by(|a, b| b.1.lifetime.total_cmp(&a.1.lifetime)).map(|x| x.0).unwrap();

                    // swap in new particle data
                    self.particles[oldest] = p;
                }
                else {
                    // push new particle
                    self.particles.push(p);
                }
            }

            self.emit_timer -= data.emit.burst_interval;
            self.burst_count += 1;
        }
    }

    pub fn update_sim(self: &mut EffectEmitterInstance, data: &EffectEmitter, parent_transform: Matrix4x4, delta: f32) {
        let xform = self.transform * parent_transform;
        let origin = Vector3::new(xform.m[3][0], xform.m[3][1], xform.m[3][2]);

        for p in &mut self.particles {
            p.position = p.position + (p.velocity * delta);
            p.angle += p.angular_velocity * delta;

            p.velocity = p.velocity + (data.accel.gravity * delta);

            let radial_dir = p.position - origin;
            if radial_dir.length_sq() > 0.0 {
                p.velocity = p.velocity + (radial_dir.normalized() * data.accel.radial_accel);
            }

            if data.accel.orbit_axis.length_sq() > 0.0 {
                let orbit_axis = data.accel.orbit_axis.normalized();
                let orbit_offset = radial_dir.dot(orbit_axis) * orbit_axis;
                let orbit_vec = radial_dir - orbit_offset;

                if orbit_vec.length_sq() > 0.0 {
                    // calculate orbit tangent vector
                    let orbit_tangent = orbit_vec.normalized().cross(orbit_axis).normalized();
                    p.velocity = p.velocity + (orbit_tangent * data.accel.orbit_accel);
                }
            }

            p.velocity = p.velocity - (p.velocity * data.accel.linear_damp);
            p.angular_velocity = p.angular_velocity - (p.angular_velocity * data.accel.angular_damp);

            p.lifetime += p.lifetime_delta * delta;
        }

        // remove any particles which have reached max lifetime
        self.particles.retain(|x| x.lifetime < 1.0);
    }

    pub fn render(self: &mut EffectEmitterInstance, r: &EffectDisplay, modelview: Matrix4x4, projection: Matrix4x4) {
        match &mut self.renderer {
            EffectEmitterRenderer::None => {
            },
            EffectEmitterRenderer::Sprite { vertices, vertex_buffer, index_buffer } => {
                let (mat, sheet, size_curve, color_curve) = match r {
                    EffectDisplay::None => unreachable!(),
                    EffectDisplay::Sprite { material, sheet, size, color } => (material, sheet, size, color),
                };

                // extract camera up, and right vectors
                let cam_up = Vector3::new(modelview.m[0][2], modelview.m[1][2], modelview.m[2][2]);
                let cam_right = Vector3::new(modelview.m[0][0], modelview.m[1][0], modelview.m[2][0]);

                vertices.clear();

                for p in &self.particles {
                    let size = size_curve.sample(p.lifetime) * p.scale;
                    let color = color_curve.sample(p.lifetime);

                    let vtx0 = p.position - (cam_right * size.x) + (cam_up * size.y);
                    let vtx1 = p.position + (cam_right * size.x) + (cam_up * size.y);
                    let vtx2 = p.position - (cam_right * size.x) - (cam_up * size.y);
                    let vtx3 = p.position + (cam_right * size.x) - (cam_up * size.y);

                    let (uv_min, uv_max) = match sheet {
                        Some(v) => {
                            let num_cells = v.rows * v.columns;
                            let sheet_index = (v.timescale * p.lifetime * num_cells as f32) as i32;

                            let column = sheet_index % (v.columns as i32);
                            let row = sheet_index / (v.columns as i32);

                            let cell_width = 1.0 / v.columns as f32;
                            let cell_height = 1.0 / v.rows as f32;

                            let min = Vector2::new(cell_width * column as f32, cell_height * row as f32);
                            let max = min + Vector2::new(cell_width, cell_height);

                            (min, max)
                        },
                        None => {
                            (Vector2::zero(), Vector2::new(1.0, 1.0))
                        },
                    };

                    vertices.push(ParticleVertex {
                        position: Vector4::new(vtx0.x, vtx0.y, vtx0.z, 1.0),
                        texcoord: Vector2::new(uv_min.x, uv_min.y),
                        color
                    });

                    vertices.push(ParticleVertex {
                        position: Vector4::new(vtx1.x, vtx1.y, vtx1.z, 1.0),
                        texcoord: Vector2::new(uv_max.x, uv_min.y),
                        color
                    });

                    vertices.push(ParticleVertex {
                        position: Vector4::new(vtx2.x, vtx2.y, vtx2.z, 1.0),
                        texcoord: Vector2::new(uv_min.x, uv_max.y),
                        color
                    });

                    vertices.push(ParticleVertex {
                        position: Vector4::new(vtx3.x, vtx3.y, vtx3.z, 1.0),
                        texcoord: Vector2::new(uv_max.x, uv_max.y),
                        color
                    });
                }

                // orphan previous buffer to eliminate sync stalls
                vertex_buffer.resize(vertex_buffer.size());
                vertex_buffer.set_data(0, &vertices);

                mat.resource.apply();

                mat.resource.shader.resource.set_uniform_mat4("mvp",
                    modelview *
                    coord_space_transform() *
                    projection
                );

                unsafe {
                    gl::FrontFace(gl::CW);

                    gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer.handle());
                    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer.handle());

                    ParticleVertex::setup_vtx_arrays(&mat.resource.shader.resource);

                    // draw geometry
                    gl::DrawElements(gl::TRIANGLES, (self.particles.len() * 6) as i32, gl::UNSIGNED_SHORT, 0 as *const _);
                }
            },
        };
    }
}

impl EffectInstance {
    pub fn new(data: &Arc<EffectData>, enable_emit: bool) -> EffectInstance {
        let emitters = data.emitters.iter().map(|x| {
            let num_particles = x.emit.max_particles as usize;

            let renderer = match &x.display {
                super::effect_data::EffectDisplay::None => EffectEmitterRenderer::None,
                super::effect_data::EffectDisplay::Sprite { .. } => {
                    let num_vertices = num_particles * 4;
                    let num_indices = num_particles * 6;

                    // index buffer can just be pre-filled with indices
                    let mut indices = Vec::with_capacity(num_indices);

                    for i in 0..num_particles {
                        let vtx_base = (i * 4) as u16;

                        indices.push(vtx_base);
                        indices.push(vtx_base + 1);
                        indices.push(vtx_base + 2);

                        indices.push(vtx_base + 1);
                        indices.push(vtx_base + 3);
                        indices.push(vtx_base + 2);
                    }

                    let mut index_buffer = Buffer::new((num_indices * size_of::<u16>()) as isize);
                    index_buffer.set_data(0, &indices);

                    EffectEmitterRenderer::Sprite {
                        vertices: Vec::with_capacity(num_vertices),
                        vertex_buffer: Buffer::new((num_vertices * size_of::<ParticleVertex>()) as isize),
                        index_buffer,
                    }
                },
            };

            EffectEmitterInstance {
                particles: Vec::with_capacity(num_particles),
                renderer,
                emit_timer: 0.0,
                burst_count: 0,
                transform: Matrix4x4::identity()
            }
        }).collect::<Vec<_>>();

        EffectInstance { transform: Matrix4x4::identity(), effect_data: data.clone(), enable_emit, emitters }
    }

    pub fn update(self: &mut EffectInstance, rng: &mut ThreadRng, delta: f32) {
        for (idx, em) in self.emitters.iter_mut().enumerate() {
            let emitter_data = &self.effect_data.emitters[idx];
            if self.enable_emit {
                em.update_emit(emitter_data, self.transform, rng, delta);
            }
            em.update_sim(emitter_data, self.transform, delta);
        }
    }

    pub fn render(self: &mut EffectInstance, modelview: Matrix4x4, projection: Matrix4x4) {
        for (idx, em) in self.emitters.iter_mut().enumerate() {
            let emitter_data = &self.effect_data.emitters[idx];
            em.render(&emitter_data.display, modelview, projection);
        }
    }
}